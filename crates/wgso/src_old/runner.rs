use crate::directive::RunDirective;
use crate::storage::Storage;
use crate::wgsl_module::WgslModule;
use crate::{Error, Program};
use futures::executor;
use fxhash::FxHashMap;
use std::path::Path;
use wgpu::{
    Adapter, BackendOptions, Backends, BindGroup, BindGroupLayout, BindGroupLayoutEntry,
    BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages, CommandEncoder,
    CommandEncoderDescriptor, ComputePass, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, Device, DeviceDescriptor, ErrorFilter, Features, Instance,
    InstanceFlags, Limits, MapMode, MemoryHints, PipelineLayoutDescriptor, PowerPreference, Queue,
    RequestAdapterOptions, ShaderModuleDescriptor, ShaderStages,
};

/// A runner to execute a WGSO program.
#[derive(Debug)]
pub struct Runner {
    program: Program,
    device: Device,
    queue: Queue,
    compute_shaders: FxHashMap<String, ComputeShaderResources>,
    compute_shader_runs: Vec<ComputeShaderRun>,
    buffers: FxHashMap<String, Buffer>,
}

impl Runner {
    /// Creates a new runner from a WGSO program directory.
    ///
    /// # Errors
    ///
    /// An error is returned if the program initialization has failed.
    pub fn new(folder_path: impl AsRef<Path>) -> Result<Self, Program> {
        let instance = Self::create_instance();
        let adapter = Self::create_adapter(&instance);
        let (device, queue) = Self::create_device(&adapter);
        let mut program = Program::parse(folder_path);
        if program.errors.is_empty() {
            device.push_error_scope(ErrorFilter::Validation);
            let buffers = program
                .storages
                .values()
                .map(|storage| (storage.name.clone(), Self::create_buffer(&device, storage)))
                .collect();
            let compute_shaders = program
                .compute_shaders
                .iter()
                .map(|(name, wgsl)| {
                    let shader = ComputeShaderResources::new(name, wgsl, &device);
                    (name.clone(), shader)
                })
                .collect::<FxHashMap<_, _>>();
            let compute_shader_runs = program
                .runs
                .iter()
                .map(|run| {
                    ComputeShaderRun::new(
                        run,
                        &program.compute_shaders[&run.name],
                        &buffers,
                        &device,
                        &compute_shaders[&run.name].layout,
                    )
                })
                .collect();
            if let Some(error) = executor::block_on(device.pop_error_scope()) {
                program.errors.push(Self::convert_wgpu_error(error));
                Err(program.with_sorted_errors())
            } else {
                Ok(Self {
                    program,
                    device,
                    queue,
                    compute_shaders,
                    compute_shader_runs,
                    buffers,
                })
            }
        } else {
            Err(program.with_sorted_errors())
        }
    }

    /// Lists all GPU buffer names.
    pub fn buffers(&self) -> impl Iterator<Item = &str> {
        self.program.storages.keys().map(String::as_str)
    }

    /// Read GPU buffer value.
    ///
    /// If the buffer doesn't exist, an empty vector is returned.
    pub fn read(&self, name: &str) -> Vec<u8> {
        if let (Some(storage), Some(buffer)) =
            (self.program.storages.get(name), self.buffers.get(name))
        {
            let read_buffer = self.device.create_buffer(&BufferDescriptor {
                label: Some("wgso:storage_read_buffer"),
                size: storage.size.into(),
                usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let mut encoder = Self::create_encoder(&self.device);
            encoder.copy_buffer_to_buffer(buffer, 0, &read_buffer, 0, storage.size.into());
            let submission_index = self.queue.submit(Some(encoder.finish()));
            let slice = read_buffer.slice(..);
            slice.map_async(MapMode::Read, |_| ());
            self.device
                .poll(wgpu::Maintain::WaitForSubmissionIndex(submission_index));
            let view = slice.get_mapped_range();
            let content = view.to_vec();
            drop(view);
            read_buffer.unmap();
            content
        } else {
            vec![]
        }
    }

    /// Runs a step of the program.
    ///
    /// # Errors
    ///
    /// An error is returned if shader execution failed.
    pub fn run_step(&mut self) -> Result<(), &Program> {
        self.device.push_error_scope(ErrorFilter::Validation);
        let mut encoder = Self::create_encoder(&self.device);
        let mut pass = Self::start_compute_pass(&mut encoder);
        for run in &self.compute_shader_runs {
            let shader = &self.compute_shaders[&run.shader_name];
            pass.set_pipeline(&shader.pipeline);
            if let Some(bind_group) = &run.bind_group {
                pass.set_bind_group(0, bind_group, &[]);
            }
            pass.dispatch_workgroups(1, 1, 1);
        }
        drop(pass);
        self.queue.submit(Some(encoder.finish()));
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

    fn create_adapter(instance: &Instance) -> Adapter {
        let adapter_request = RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: None,
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
        };
        executor::block_on(adapter.request_device(&device_descriptor, None))
            .expect("error when retrieving graphic device")
    }

    fn create_buffer(device: &Device, storage: &Storage) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some(&format!("`var<storage, _> {}`", storage.name)),
            size: storage.size.into(),
            usage: BufferUsages::STORAGE
                | BufferUsages::COPY_SRC
                | BufferUsages::UNIFORM
                | BufferUsages::VERTEX
                | BufferUsages::INDEX,
            mapped_at_creation: false,
        })
    }

    fn create_encoder(device: &Device) -> CommandEncoder {
        device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("wgso:encoder"),
        })
    }

    pub(crate) fn start_compute_pass(encoder: &mut CommandEncoder) -> ComputePass<'_> {
        encoder.begin_compute_pass(&ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        })
    }
}

#[derive(Debug)]
struct ComputeShaderResources {
    pipeline: ComputePipeline,
    layout: BindGroupLayout,
}

impl ComputeShaderResources {
    fn new(name: &str, module: &WgslModule, device: &Device) -> Self {
        let layout = Self::create_bind_group_layout(name, module, device);
        let pipeline = Self::create_pipeline(name, module, device, &layout);
        Self { pipeline, layout }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn create_bind_group_layout(
        name: &str,
        module: &WgslModule,
        device: &Device,
    ) -> BindGroupLayout {
        let storage_entries =
            module
                .storage_bindings
                .values()
                .map(|binding| BindGroupLayoutEntry {
                    binding: binding.index as u32,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                });
        let uniform_entries =
            module
                .uniform_bindings
                .values()
                .map(|binding| BindGroupLayoutEntry {
                    binding: binding.index as u32,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                });
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("#shader<compute> {name}")),
            entries: &storage_entries.chain(uniform_entries).collect::<Vec<_>>(),
        })
    }

    fn create_pipeline(
        name: &str,
        module: &WgslModule,
        device: &Device,
        layout: &BindGroupLayout,
    ) -> ComputePipeline {
        let label = format!("#shader<compute> {name}");
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&label),
            source: wgpu::ShaderSource::Wgsl(module.cleaned_code.as_str().into()),
        });
        device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some(&label),
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some(&label),
                bind_group_layouts: &[layout],
                push_constant_ranges: &[],
            })),
            module: &module,
            entry_point: None,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
    }
}

#[derive(Debug)]
struct ComputeShaderRun {
    shader_name: String,
    bind_group: Option<BindGroup>,
}

impl ComputeShaderRun {
    fn new(
        directive: &RunDirective,
        module: &WgslModule,
        buffers: &FxHashMap<String, Buffer>,
        device: &Device,
        layout: &BindGroupLayout,
    ) -> Self {
        let bind_group = (!module.storage_bindings.is_empty())
            .then(|| Self::create_bind_group(directive, module, buffers, device, layout));
        Self {
            shader_name: directive.name.clone(),
            bind_group,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn create_bind_group(
        directive: &RunDirective,
        module: &WgslModule,
        buffers: &FxHashMap<String, Buffer>,
        device: &Device,
        layout: &BindGroupLayout,
    ) -> BindGroup {
        let storage_entries =
            module
                .storage_bindings
                .iter()
                .map(|(name, binding)| wgpu::BindGroupEntry {
                    binding: binding.index as u32,
                    resource: buffers[name].as_entire_binding(),
                });
        let uniform_entries =
            module
                .uniform_bindings
                .iter()
                .map(|(name, binding)| wgpu::BindGroupEntry {
                    binding: binding.index as u32,
                    resource: buffers[&directive.params[name].value].as_entire_binding(),
                });
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("#{}", directive.source)),
            layout,
            entries: &storage_entries.chain(uniform_entries).collect::<Vec<_>>(),
        })
    }
}
