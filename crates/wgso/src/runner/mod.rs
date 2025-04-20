use crate::fields::StorageField;
use crate::runner::target::{Target, TargetConfig, TargetSpecialized, TextureTarget, WindowTarget};
use crate::{Error, Program};
use futures::executor;
use fxhash::FxHashMap;
use shader::{ComputeShaderResources, ComputeShaderRun};
use std::path::Path;
use std::sync::Arc;
use wgpu::{
    Adapter, BackendOptions, Backends, Buffer, BufferDescriptor, BufferUsages, Color,
    CommandEncoder, CommandEncoderDescriptor, ComputePass, ComputePassDescriptor, Device,
    DeviceDescriptor, ErrorFilter, Extent3d, Features, Instance, InstanceFlags, Limits, LoadOp,
    MapMode, MemoryHints, Operations, PollType, PowerPreference, Queue, RenderPass,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration, SurfaceTexture,
    TexelCopyBufferInfo, TexelCopyBufferLayout, Texture, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureView, TextureViewDescriptor, Trace,
};
use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

mod shader;
mod target;

/// A runner to execute a WGSO program.
#[derive(Debug)]
pub struct Runner {
    target: Target,
    instance: Instance,
    device: Device,
    adapter: Adapter,
    queue: Queue,
    program: Program,
    compute_shaders: FxHashMap<String, ComputeShaderResources>,
    compute_shader_runs: Vec<ComputeShaderRun>,
    buffers: FxHashMap<String, Buffer>,
    is_initialized: bool,
}

impl Runner {
    /// Creates a new runner from a WGSO program directory.
    ///
    /// # Errors
    ///
    /// An error is returned if the program initialization has failed.
    pub fn new(
        folder_path: impl AsRef<Path>,
        event_loop: Option<&ActiveEventLoop>,
        size: Option<(u32, u32)>,
    ) -> Result<Self, Program> {
        let target = TargetConfig {
            size: size.unwrap_or((800, 600)),
        };
        let instance = Self::create_instance();
        let window_surface = event_loop.map(|event_loop| {
            // coverage: off (window cannot be tested)
            let window = Self::create_window(event_loop, target.size);
            let surface = Self::create_surface(&instance, window.clone());
            (window, surface)
        }); // coverage: on
        let adapter = Self::create_adapter(&instance, window_surface.as_ref());
        let (device, queue) = Self::create_device(&adapter);
        let surface_config = window_surface.as_ref().map(|(_, surface)| {
            // coverage: off (window cannot be tested)
            Self::create_surface_config(&adapter, &device, surface, target.size)
        }); // coverage: on
        let depth_buffer = Self::create_depth_buffer(&device, target.size);
        let mut program = Program::parse(folder_path);
        if program.errors.is_empty() {
            device.push_error_scope(ErrorFilter::Validation);
            let buffers = Self::create_buffers(&device, &program);
            let compute_shaders = Self::create_compute_shaders(&device, &program);
            let compute_shader_runs =
                Self::create_compute_shader_runs(&device, &program, &buffers, &compute_shaders);
            if let Some(error) = executor::block_on(device.pop_error_scope()) {
                program.errors.push(Self::convert_wgpu_error(error));
                Err(program.with_sorted_errors())
            } else {
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
                    let texture = Self::create_target_texture(&device, target.size);
                    let view = texture.create_view(&TextureViewDescriptor::default());
                    Target {
                        inner: TargetSpecialized::Texture(TextureTarget { texture, view }),
                        config: target,
                        depth_buffer,
                    }
                };
                Ok(Self {
                    target,
                    program,
                    device,
                    adapter,
                    queue,
                    compute_shaders,
                    compute_shader_runs,
                    buffers,
                    is_initialized: false,
                    instance,
                })
            }
        } else {
            Err(program.with_sorted_errors())
        }
    }

    /// Lists all GPU buffer names.
    pub fn buffers(&self) -> impl Iterator<Item = &str> {
        self.program.resources.storages.keys().map(String::as_str)
    }

    /// Read GPU buffer value.
    ///
    /// If the buffer doesn't exist, an empty vector is returned.
    /// Inner fields can also be provided (e.g. `my_buffer.field.inner`).
    pub fn read(&self, path: &str) -> Vec<u8> {
        let Some(field) = StorageField::parse(&self.program, path) else {
            return vec![];
        };
        let read_buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some("wgso:storage_read_buffer"),
            size: field.type_.size.into(),
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self.create_encoder();
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
                let padded_bytes_per_row = Self::calculate_padded_row_bytes(size.0);
                let padded_row_bytes = Self::calculate_padded_row_bytes(size.0);
                let tmp_buffer = self.device.create_buffer(&BufferDescriptor {
                    label: Some("wgso:target_read_buffer"),
                    size: (padded_bytes_per_row * size.1).into(),
                    usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                let mut encoder = self.create_encoder();
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
                let padded_row_bytes = Self::calculate_padded_row_bytes(size.0);
                let unpadded_row_bytes = Self::calculate_unpadded_row_bytes(size.0);
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
        let mut encoder = self.create_encoder();
        let pass = Self::start_compute_pass(&mut encoder);
        self.run_compute_step(pass);
        match &self.target.inner {
            // coverage: off (window cannot be tested)
            TargetSpecialized::Window(target) => {
                let texture = target.create_surface_texture();
                let view = Self::create_surface_view(&texture);
                let pass = Self::create_render_pass(&mut encoder, &view, &self.target.depth_buffer);
                self.run_draw_step(pass);
                self.queue.submit(Some(encoder.finish()));
                texture.present();
            }
            // coverage: on
            TargetSpecialized::Texture(target) => {
                let pass =
                    Self::create_render_pass(&mut encoder, &target.view, &self.target.depth_buffer);
                self.run_draw_step(pass);
                self.queue.submit(Some(encoder.finish()));
            }
        }
        self.is_initialized = true;
        if let Some(error) = executor::block_on(self.device.pop_error_scope()) {
            self.program.errors.push(Self::convert_wgpu_error(error));
            Err(&self.program)
        } else {
            Ok(())
        }
    }

    fn convert_wgpu_error(error: wgpu::Error) -> Error {
        Error::WgpuValidation(match error {
            wgpu::Error::Validation { description, .. } => description,
            wgpu::Error::OutOfMemory { .. } | wgpu::Error::Internal { .. } => {
                unreachable!("internal error: WGPU error should be for validation")
            }
        })
    }

    fn create_instance() -> Instance {
        Instance::new(&wgpu::InstanceDescriptor {
            backends: Backends::from_env().unwrap_or_else(Backends::all),
            flags: InstanceFlags::default(),
            backend_options: BackendOptions::default(),
        })
    }

    fn create_adapter(
        instance: &Instance,
        window_surface: Option<&(Arc<Window>, Surface<'_>)>,
    ) -> Adapter {
        let adapter_request = RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: window_surface.map(|(_, surface)| surface),
        };
        executor::block_on(instance.request_adapter(&adapter_request))
            .expect("no supported graphic adapter found")
    }

    fn create_device(adapter: &Adapter) -> (Device, Queue) {
        let device_descriptor = DeviceDescriptor {
            label: Some("wgso:device"),
            required_features: Features::default(),
            required_limits: Limits::default(),
            memory_hints: MemoryHints::Performance,
            trace: Trace::Off,
        };
        executor::block_on(adapter.request_device(&device_descriptor))
            .expect("error when retrieving graphic device")
    }

    fn create_buffer(device: &Device, name: &str, size: u64) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some(&format!("`var<storage, _> {name}`")),
            size,
            usage: BufferUsages::STORAGE
                | BufferUsages::COPY_SRC
                | BufferUsages::UNIFORM
                | BufferUsages::VERTEX
                | BufferUsages::INDEX,
            mapped_at_creation: false,
        })
    }

    fn create_encoder(&self) -> CommandEncoder {
        self.device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("wgso:encoder"),
            })
    }

    fn start_compute_pass(encoder: &mut CommandEncoder) -> ComputePass<'_> {
        encoder.begin_compute_pass(&ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        })
    }

    fn create_target_texture(device: &Device, size: (u32, u32)) -> Texture {
        device.create_texture(&TextureDescriptor {
            label: Some("wgso:target_texture"),
            size: Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    }

    fn create_depth_buffer(device: &Device, size: (u32, u32)) -> TextureView {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("wgso:depth_texture"),
            size: Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        texture.create_view(&TextureViewDescriptor::default())
    }

    fn create_render_pass<'a>(
        encoder: &'a mut CommandEncoder,
        view: &'a TextureView,
        depth_buffer: &'a TextureView,
    ) -> RenderPass<'a> {
        encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("wgso:render_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: depth_buffer,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }

    fn create_buffers(device: &Device, program: &Program) -> FxHashMap<String, Buffer> {
        program
            .resources
            .storages
            .iter()
            .map(|(name, type_)| {
                let size = type_.size.into();
                (name.clone(), Self::create_buffer(device, name, size))
            })
            .collect()
    }

    fn create_compute_shaders(
        device: &Device,
        program: &Program,
    ) -> FxHashMap<String, ComputeShaderResources> {
        program
            .resources
            .compute_shaders
            .iter()
            .map(|(name, (directive, module))| {
                let shader = ComputeShaderResources::new(name, directive, module, device);
                (name.clone(), shader)
            })
            .collect()
    }

    fn create_compute_shader_runs(
        device: &Device,
        program: &Program,
        buffers: &FxHashMap<String, Buffer>,
        compute_shaders: &FxHashMap<String, ComputeShaderResources>,
    ) -> Vec<ComputeShaderRun> {
        program
            .resources
            .runs
            .iter()
            .map(|directive| {
                ComputeShaderRun::new(
                    program,
                    directive,
                    buffers,
                    device,
                    compute_shaders[&directive.name.label].layout.as_ref(),
                )
            })
            .collect()
    }

    fn calculate_padded_row_bytes(width: u32) -> u32 {
        let unpadded_bytes_per_row = Self::calculate_unpadded_row_bytes(width);
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        unpadded_bytes_per_row + padded_bytes_per_row_padding
    }

    #[allow(clippy::cast_possible_truncation)]
    fn calculate_unpadded_row_bytes(width: u32) -> u32 {
        let bytes_per_pixel = size_of::<u32>() as u32;
        width * bytes_per_pixel
    }

    pub(crate) fn run_compute_step(&self, mut pass: ComputePass<'_>) {
        for run in &self.compute_shader_runs {
            if !run.is_init || !self.is_initialized {
                let shader = &self.compute_shaders[&run.shader_name];
                pass.set_pipeline(&shader.pipeline);
                if let Some(bind_group) = &run.bind_group {
                    pass.set_bind_group(0, bind_group, &[]);
                }
                pass.dispatch_workgroups(
                    shader.directive.workgroup_count_x.into(),
                    shader.directive.workgroup_count_y.into(),
                    shader.directive.workgroup_count_z.into(),
                );
            }
        }
    }

    #[allow(clippy::unused_self)]
    pub(crate) fn run_draw_step(&self, _pass: RenderPass<'_>) {
        // do nothing for the moment
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
                target.surface = Self::create_surface(&self.instance, target.window.clone());
                target.surface_config = Self::create_surface_config(
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
                    Self::create_depth_buffer(&self.device, self.target.config.size);
                target.surface_config = Self::create_surface_config(
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

    fn create_window(event_loop: &ActiveEventLoop, size: (u32, u32)) -> Arc<Window> {
        let size = PhysicalSize::new(size.0, size.1);
        let window = event_loop
            .create_window(Window::default_attributes().with_inner_size(size))
            .expect("cannot create window");
        Arc::new(window)
    }

    fn create_surface(instance: &Instance, window: Arc<Window>) -> Surface<'static> {
        instance
            .create_surface(window)
            .expect("cannot create surface")
    }

    fn create_surface_config(
        adapter: &Adapter,
        device: &Device,
        surface: &Surface<'_>,
        size: (u32, u32),
    ) -> SurfaceConfiguration {
        let config = surface
            .get_default_config(adapter, size.0, size.1)
            .expect("not supported surface");
        surface.configure(device, &config);
        config
    }

    fn create_surface_view(texture: &SurfaceTexture) -> TextureView {
        texture
            .texture
            .create_view(&TextureViewDescriptor::default())
    }

    // coverage: on
}
