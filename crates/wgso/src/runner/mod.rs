use crate::fields::StorageField;
use crate::{Error, Program};
use futures::executor;
use fxhash::FxHashMap;
use shader::{ComputeShaderResources, ComputeShaderRun};
use std::path::Path;
use wgpu::{
    Adapter, BackendOptions, Backends, Buffer, BufferDescriptor, BufferUsages, CommandEncoder,
    CommandEncoderDescriptor, ComputePass, ComputePassDescriptor, Device, DeviceDescriptor,
    ErrorFilter, Features, Instance, InstanceFlags, Limits, MapMode, MemoryHints, PollType,
    PowerPreference, Queue, RequestAdapterOptions, Trace,
};

mod shader;

/// A runner to execute a WGSO program.
#[derive(Debug)]
pub struct Runner {
    program: Program,
    device: Device,
    queue: Queue,
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
    pub fn new(folder_path: impl AsRef<Path>) -> Result<Self, Program> {
        let instance = Self::create_instance();
        let adapter = Self::create_adapter(&instance);
        let (device, queue) = Self::create_device(&adapter);
        let mut program = Program::parse(folder_path);
        if program.errors.is_empty() {
            device.push_error_scope(ErrorFilter::Validation);
            let buffers = program
                .resources
                .storages
                .iter()
                .map(|(name, type_)| {
                    let size = type_.size.into();
                    (name.clone(), Self::create_buffer(&device, name, size))
                })
                .collect();
            let compute_shaders = program
                .resources
                .compute_shaders
                .iter()
                .map(|(name, (directive, module))| {
                    let shader = ComputeShaderResources::new(name, directive, module, &device);
                    (name.clone(), shader)
                })
                .collect::<FxHashMap<_, _>>();
            let compute_shader_runs = program
                .resources
                .runs
                .iter()
                .map(|(directive, _)| {
                    ComputeShaderRun::new(
                        &program,
                        directive,
                        &buffers,
                        &device,
                        &compute_shaders[&directive.name.label].layout,
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
                    is_initialized: false,
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
        let mut encoder = Self::create_encoder(&self.device);
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
            if !run.is_init || !self.is_initialized {
                let shader = &self.compute_shaders[&run.shader_name];
                pass.set_pipeline(&shader.pipeline);
                if let Some(bind_group) = &run.bind_group {
                    pass.set_bind_group(0, bind_group, &[]);
                }
                pass.dispatch_workgroups(
                    shader.directive.workgroup_count_x,
                    shader.directive.workgroup_count_y,
                    shader.directive.workgroup_count_z,
                );
            }
        }
        self.is_initialized = true;
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
