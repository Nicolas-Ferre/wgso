use crate::program::file::SourceFolder;
use crate::runner::shaders::RenderShaderResources;
use crate::runner::std::StdState;
use crate::runner::target::{Target, TargetConfig, TargetSpecialized, TextureTarget, WindowTarget};
use crate::{Error, Program};
use ::std::path::PathBuf;
use futures::executor;
use fxhash::FxHashMap;
use shader_execution::ShaderExecution;
use shaders::ComputeShaderResources;
use watcher::RunnerWatcher;
use wgpu::{
    Adapter, Buffer, BufferDescriptor, BufferUsages, ComputePass, Device, ErrorFilter, Extent3d,
    Instance, MapMode, PollType, Queue, RenderPass, TexelCopyBufferInfo, TexelCopyBufferLayout,
    TextureFormat, TextureViewDescriptor,
};
use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;

mod gpu;
mod shader_execution;
mod shaders;
mod std;
mod target;
mod watcher;

/// A runner to execute a WGSO program.
#[derive(Debug)]
pub struct Runner {
    pub(crate) std_state: StdState,
    target: Target,
    instance: Instance,
    device: Device,
    adapter: Adapter,
    queue: Queue,
    program: Program,
    compute_shaders: FxHashMap<(PathBuf, String), ComputeShaderResources>,
    render_shaders: FxHashMap<(PathBuf, String), RenderShaderResources>,
    compute_shader_executions: Vec<ShaderExecution>,
    render_shader_executions: Vec<ShaderExecution>,
    buffers: FxHashMap<String, Buffer>,
    is_initialized: bool,
    watcher: RunnerWatcher,
}

impl Runner {
    /// Creates a new runner from a WGSO program directory.
    ///
    /// # Errors
    ///
    /// An error is returned if the program initialization has failed.
    pub fn new(
        source: impl SourceFolder,
        event_loop: Option<&ActiveEventLoop>,
        size: Option<(u32, u32)>,
    ) -> Result<Self, Program> {
        let folder_path = source.path();
        let program = Program::parse(source);
        if !program.errors.is_empty() {
            return Err(program.with_sorted_errors());
        }
        let target = TargetConfig {
            size: size.unwrap_or((800, 600)),
        };
        let instance = gpu::create_instance();
        let window_surface = event_loop.map(|event_loop| {
            // coverage: off (window cannot be tested)
            let window = gpu::create_window(event_loop, target.size);
            let surface = gpu::create_surface(&instance, window.clone());
            (window, surface)
        }); // coverage: on
        let adapter = gpu::create_adapter(
            &instance,
            window_surface.as_ref().map(|(_, surface)| surface),
        );
        let (device, queue) = gpu::create_device(&adapter);
        let surface_config = window_surface.as_ref().map(|(_, surface)| {
            // coverage: off (window cannot be tested)
            gpu::create_surface_config(&adapter, &device, surface, target.size)
        }); // coverage: on
        let depth_buffer = gpu::create_depth_buffer(&device, target.size);
        let target = if let (Some((window, surface)), Some(surface_config)) =
            (window_surface, surface_config)
        {
            // coverage: off (window cannot be tested)
            Target {
                inner: TargetSpecialized::Window(WindowTarget {
                    window,
                    surface,
                    surface_config,
                }),
                config: target,
                depth_buffer,
            }
            // coverage: on
        } else {
            let texture = gpu::create_target_texture(&device, target.size);
            let view = texture.create_view(&TextureViewDescriptor::default());
            Target {
                inner: TargetSpecialized::Texture(TextureTarget { texture, view }),
                config: target,
                depth_buffer,
            }
        };
        let buffers = Self::create_buffers(&device, &program);
        let mut runner = Self {
            std_state: StdState::default(),
            target,
            program,
            device,
            adapter,
            queue,
            compute_shaders: FxHashMap::default(),
            render_shaders: FxHashMap::default(),
            compute_shader_executions: vec![],
            render_shader_executions: vec![],
            buffers,
            is_initialized: false,
            instance,
            watcher: RunnerWatcher::new(&folder_path),
        };
        if runner.load_shaders(None) {
            Ok(runner)
        } else {
            Err(runner.program.with_sorted_errors())
        }
    }

    /// Returns the time of the last executed frame.
    pub fn delta_secs(&self) -> f32 {
        self.std_state.time.frame_delta_secs
    }

    /// Lists all GPU buffer names.
    pub fn buffers(&self) -> impl Iterator<Item = &str> {
        self.program.modules.storages.keys().map(String::as_str)
    }

    /// Writes GPU buffer data.
    ///
    /// If the buffer doesn't exist, nothing happens.
    /// Inner fields can also be provided (e.g. `my_buffer.field.inner`).
    ///
    /// # Panics
    ///
    /// This will panic if the `data` length doesn't match the buffer size.
    pub fn write(&self, path: &str, data: &[u8]) {
        let Some(field) = self.program.parse_field(path) else {
            return;
        };
        assert_eq!(data.len(), field.type_.size as usize, "incorrect data size");
        self.queue.write_buffer(
            &self.buffers[&field.buffer_name],
            field.type_.offset.into(),
            data,
        );
    }

    /// Reads GPU buffer data.
    ///
    /// If the buffer doesn't exist, an empty vector is returned.
    /// Inner fields can also be provided (e.g. `my_buffer.field.inner`).
    pub fn read(&self, path: &str) -> Vec<u8> {
        let Some(field) = self.program.parse_field(path) else {
            return vec![];
        };
        let read_buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some("wgso:storage_read_buffer"),
            size: field.type_.size.into(),
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = gpu::create_encoder(&self.device);
        encoder.copy_buffer_to_buffer(
            &self.buffers[&field.buffer_name],
            field.type_.offset.into(),
            &read_buffer,
            0,
            field.type_.size.into(),
        );
        let submission_index = self.queue.submit(Some(encoder.finish()));
        let slice = read_buffer.slice(..);
        slice.map_async(MapMode::Read, |_| ());
        self.device
            .poll(PollType::WaitForSubmissionIndex(submission_index))
            .expect("cannot read buffer");
        let view = slice.get_mapped_range();
        let content = view.to_vec();
        drop(view);
        read_buffer.unmap();
        content
    }

    /// Read texture target.
    ///
    /// If the surface is not a texture, an empty vector is returned.
    pub fn read_target(&self) -> Vec<u8> {
        match &self.target.inner {
            TargetSpecialized::Texture(target) => {
                let size = self.target.config.size;
                let (padded_row_bytes, _) = gpu::padded_unpadded_row_bytes(size.0);
                let tmp_buffer = self.device.create_buffer(&BufferDescriptor {
                    label: Some("wgso:target_read_buffer"),
                    size: (padded_row_bytes * size.1).into(),
                    usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                let mut encoder = gpu::create_encoder(&self.device);
                encoder.copy_texture_to_buffer(
                    target.texture.as_image_copy(),
                    TexelCopyBufferInfo {
                        buffer: &tmp_buffer,
                        layout: TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(padded_row_bytes),
                            rows_per_image: None,
                        },
                    },
                    Extent3d {
                        width: size.0,
                        height: size.1,
                        depth_or_array_layers: 1,
                    },
                );
                let submission_index = self.queue.submit(Some(encoder.finish()));
                let slice = tmp_buffer.slice(..);
                slice.map_async(MapMode::Read, |_| ());
                self.device
                    .poll(PollType::WaitForSubmissionIndex(submission_index))
                    .expect("cannot read target buffer");
                let view = slice.get_mapped_range();
                let (padded_row_bytes, unpadded_row_bytes) = gpu::padded_unpadded_row_bytes(size.0);
                let content = view
                    .chunks(padded_row_bytes as usize)
                    .flat_map(|a| &a[..unpadded_row_bytes as usize])
                    .copied()
                    .collect();
                drop(view);
                tmp_buffer.unmap();
                content
            }
            TargetSpecialized::Window(_) => vec![], // no-coverage (window cannot be tested)
        }
    }

    /// Runs a step of the program.
    ///
    /// # Errors
    ///
    /// An error is returned if shader execution failed.
    pub fn run_step(&mut self) -> Result<(), &Program> {
        self.device.push_error_scope(ErrorFilter::Validation);
        if !self.is_initialized {
            self.std_state.update(self.target.config.size);
        }
        self.write_std_state();
        let mut encoder = gpu::create_encoder(&self.device);
        let pass = gpu::start_compute_pass(&mut encoder);
        self.run_compute_step(pass);
        match &self.target.inner {
            // coverage: off (window cannot be tested)
            TargetSpecialized::Window(target) => {
                let texture = target.create_surface_texture();
                let view = gpu::create_surface_view(&texture);
                let pass = gpu::create_render_pass(&mut encoder, &view, &self.target.depth_buffer);
                self.run_draw_step(pass);
                self.queue.submit(Some(encoder.finish()));
                texture.present();
            }
            // coverage: on
            TargetSpecialized::Texture(target) => {
                let pass =
                    gpu::create_render_pass(&mut encoder, &target.view, &self.target.depth_buffer);
                self.run_draw_step(pass);
                self.queue.submit(Some(encoder.finish()));
            }
        }
        self.std_state.update(self.target.config.size);
        if let Some(error) = executor::block_on(self.device.pop_error_scope()) {
            self.program.errors.push(gpu::convert_error(error));
            Err(&self.program)
        } else {
            Ok(())
        }
    }

    /// Reloads the runner if a file in the program directory has been updated.
    ///
    /// # Errors
    ///
    /// An error is returned if the program cannot be reloaded.
    pub fn reload_on_change(&mut self) -> Result<(), Program> {
        if !self.watcher.detect_changes() {
            return Ok(());
        }
        let mut program = Program::parse(self.program.root_path.as_path());
        if !program.errors.is_empty() {
            return Err(program.with_sorted_errors());
        }
        if program.modules.storages != self.program.modules.storages {
            program.errors.push(Error::ChangedStorageStructure);
            return Err(program.with_sorted_errors());
        }
        if self.load_shaders(Some(&mut program)) {
            self.program = program;
            Ok(())
        } else {
            Err(program.with_sorted_errors())
        }
    }

    fn write_std_state(&self) {
        self.write("std_.time", &self.std_state.time.data());
        self.write("std_.surface", &self.std_state.surface.data());
        self.write("std_.keyboard", &self.std_state.keyboard.data());
        self.write("std_.mouse", &self.std_state.mouse.data());
    }

    fn load_shaders(&mut self, program: Option<&mut Program>) -> bool {
        let program = program.unwrap_or(&mut self.program);
        self.device.push_error_scope(ErrorFilter::Validation);
        let compute_shaders = Self::create_compute_shaders(&self.device, program);
        let render_shaders =
            Self::create_render_shaders(&self.device, program, self.target.texture_format());
        let compute_shader_executions = Self::create_compute_shader_runs(
            &self.device,
            program,
            &self.buffers,
            &compute_shaders,
        );
        let render_shader_executions =
            Self::create_render_shader_draws(&self.device, program, &self.buffers, &render_shaders);
        if let Some(error) = executor::block_on(self.device.pop_error_scope()) {
            program.errors.push(gpu::convert_error(error));
            false
        } else {
            self.compute_shaders = compute_shaders;
            self.render_shaders = render_shaders;
            self.compute_shader_executions = compute_shader_executions;
            self.render_shader_executions = render_shader_executions;
            true
        }
    }

    fn create_buffers(device: &Device, program: &Program) -> FxHashMap<String, Buffer> {
        program
            .modules
            .storages
            .iter()
            .map(|(name, type_)| {
                let size = type_.size.into();
                (
                    name.clone(),
                    gpu::create_buffer(device, &format!("`var<storage, _> {name}`"), size),
                )
            })
            .collect()
    }

    fn create_compute_shaders(
        device: &Device,
        program: &Program,
    ) -> FxHashMap<(PathBuf, String), ComputeShaderResources> {
        program
            .modules
            .compute
            .iter()
            .map(|(name, module)| {
                let shader = ComputeShaderResources::new(module, device);
                (name.clone(), shader)
            })
            .collect()
    }

    fn create_render_shaders(
        device: &Device,
        program: &Program,
        texture_format: TextureFormat,
    ) -> FxHashMap<(PathBuf, String), RenderShaderResources> {
        program
            .modules
            .render
            .iter()
            .map(|(name, module)| {
                let shader = RenderShaderResources::new(module, texture_format, device);
                (name.clone(), shader)
            })
            .collect()
    }

    fn create_compute_shader_runs(
        device: &Device,
        program: &Program,
        buffers: &FxHashMap<String, Buffer>,
        compute_shaders: &FxHashMap<(PathBuf, String), ComputeShaderResources>,
    ) -> Vec<ShaderExecution> {
        program
            .sections
            .run_directives()
            .map(|directive| {
                ShaderExecution::new(
                    program,
                    directive,
                    buffers,
                    device,
                    compute_shaders[&directive.item_ident(&program.root_path)]
                        .layout
                        .as_ref(),
                )
            })
            .collect()
    }

    fn create_render_shader_draws(
        device: &Device,
        program: &Program,
        buffers: &FxHashMap<String, Buffer>,
        render_shaders: &FxHashMap<(PathBuf, String), RenderShaderResources>,
    ) -> Vec<ShaderExecution> {
        program
            .sections
            .draw_directives()
            .map(|directive| {
                ShaderExecution::new(
                    program,
                    directive,
                    buffers,
                    device,
                    render_shaders[&directive.item_ident(&program.root_path)]
                        .layout
                        .as_ref(),
                )
            })
            .collect()
    }

    fn run_compute_step(&mut self, mut pass: ComputePass<'_>) {
        for run in &self.compute_shader_executions {
            if !run.is_init || !self.is_initialized {
                let shader = &self.compute_shaders[&run.shader_ident];
                pass.set_pipeline(&shader.pipeline);
                if let Some(bind_group) = &run.bind_group {
                    pass.set_bind_group(0, bind_group, &[]);
                }
                let workgroup_count = shader.directive.workgroup_count();
                pass.dispatch_workgroups(
                    workgroup_count.0.into(),
                    workgroup_count.1.into(),
                    workgroup_count.2.into(),
                );
            }
        }
        self.is_initialized = true;
    }

    fn run_draw_step(&self, mut pass: RenderPass<'_>) {
        for draw in &self.render_shader_executions {
            let shader = &self.render_shaders[&draw.shader_ident];
            pass.set_pipeline(&shader.pipeline);
            if let Some(bind_group) = &draw.bind_group {
                pass.set_bind_group(0, bind_group, &[]);
            }
            let vertex_count = self.bind_buffer(&mut pass, draw, 0, true);
            let instance_count = self.bind_buffer(&mut pass, draw, 1, false);
            pass.draw(0..vertex_count, 0..instance_count);
        }
    }

    #[allow(clippy::cast_lossless)]
    fn bind_buffer(
        &self,
        pass: &mut RenderPass<'_>,
        draw: &ShaderExecution,
        slot: u32,
        is_vertex: bool,
    ) -> u32 {
        let buffer_arg = if is_vertex {
            draw.directive.vertex_buffer()
        } else {
            draw.directive.instance_buffer()
        };
        let buffer_name = &buffer_arg.var.slice;
        let storage = &self.program.modules.storages[buffer_name];
        let buffer = &self.buffers[buffer_name];
        let field_type = storage
            .field_ident_type(&buffer_arg.fields)
            .expect("internal error: buffer fields should be validated");
        pass.set_vertex_buffer(
            slot,
            buffer.slice(field_type.offset as u64..(field_type.offset + field_type.size) as u64),
        );
        field_type.array_params.as_ref().map_or(1, |(_, len)| *len)
    }

    // coverage: off (window cannot be tested)

    /// Requests window surface redraw.
    ///
    /// # Panics
    ///
    /// This will panic if the surface is not a window.
    pub fn request_redraw(&self) {
        match &self.target.inner {
            TargetSpecialized::Window(target) => target.window.request_redraw(),
            TargetSpecialized::Texture(_) => {
                unreachable!("surface should be a window")
            }
        }
    }

    /// Refreshes the rendering surface.
    pub fn refresh_surface(&mut self) {
        match &mut self.target.inner {
            TargetSpecialized::Window(target) => {
                target.surface = gpu::create_surface(&self.instance, target.window.clone());
                target.surface_config = gpu::create_surface_config(
                    &self.adapter,
                    &self.device,
                    &target.surface,
                    self.target.config.size,
                );
            }
            TargetSpecialized::Texture(_) => {
                unreachable!("internal error: refreshing non-window target surface")
            }
        }
    }

    /// Resizes rendering surface.
    pub fn update_surface_size(&mut self, size: PhysicalSize<u32>) {
        match &mut self.target.inner {
            TargetSpecialized::Window(target) => {
                self.target.config.size = (size.width.max(1), size.height.max(1));
                self.target.depth_buffer =
                    gpu::create_depth_buffer(&self.device, self.target.config.size);
                target.surface_config = gpu::create_surface_config(
                    &self.adapter,
                    &self.device,
                    &target.surface,
                    self.target.config.size,
                );
            }
            TargetSpecialized::Texture(_) => {
                unreachable!("internal error: updating non-window target surface")
            }
        }
    }

    // coverage: on
}
