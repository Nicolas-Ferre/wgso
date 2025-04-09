use crate::storage::Storage;
use crate::Program;
use futures::executor;
use fxhash::FxHashMap;
use std::path::Path;
use wgpu::{
    Adapter, BackendOptions, Backends, Buffer, BufferDescriptor, BufferUsages,
    CommandEncoderDescriptor, Device, DeviceDescriptor, Features, Instance, InstanceFlags, Limits,
    MapMode, MemoryHints, PowerPreference, Queue, RequestAdapterOptions,
};

/// A runner to execute a WGSO program.
#[derive(Debug)]
pub struct Runner {
    program: Program,
    device: Device,
    queue: Queue,
    buffers: FxHashMap<String, Buffer>,
}

impl Runner {
    /// Creates a new runner from a WGSO program directory.
    ///
    /// # Errors
    ///
    /// An error is returned if the parsing has failed.
    #[allow(clippy::result_large_err)]
    pub fn new(folder_path: impl AsRef<Path>) -> Result<Self, Program> {
        let instance = Self::create_instance();
        let adapter = Self::create_adapter(&instance);
        let (device, queue) = Self::create_device(&adapter);
        let program = Program::parse(folder_path);
        if program.errors.is_empty() {
            Ok(Self {
                buffers: program
                    .storages
                    .values()
                    .map(|storage| (storage.name.clone(), Self::create_buffer(&device, storage)))
                    .collect(),
                device,
                queue,
                program,
            })
        } else {
            Err(program)
        }
    }

    /// Lists all GPU buffer names.
    pub fn buffers(&self) -> impl Iterator<Item = &str> {
        self.program.storages.keys().map(String::as_str)
    }

    /// Read GPU buffer value.
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
            let mut encoder = self
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("wgso:storage_read_encoder"),
                });
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
            label: Some("wgso:buffer"),
            size: storage.size.into(),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        })
    }
}
