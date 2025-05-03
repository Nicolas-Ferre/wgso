use crate::Error;
use futures::executor;
use std::sync::Arc;
use wgpu::{
    Adapter, BackendOptions, Backends, BindGroupLayout, Buffer, BufferDescriptor, BufferUsages,
    Color, CommandEncoder, CommandEncoderDescriptor, ComputePass, ComputePassDescriptor, Device,
    DeviceDescriptor, Extent3d, Features, Instance, InstanceFlags, Limits, LoadOp, MemoryHints,
    Operations, PipelineLayout, PipelineLayoutDescriptor, PowerPreference, Queue, RenderPass,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration, SurfaceTexture, Texture,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor, Trace,
};
use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

pub(crate) fn convert_error(error: wgpu::Error) -> Error {
    Error::WgpuValidation(match error {
        wgpu::Error::Validation { description, .. } => description,
        wgpu::Error::OutOfMemory { .. } | wgpu::Error::Internal { .. } => {
            unreachable!("internal error: WGPU error should be for validation")
        }
    })
}

#[allow(clippy::cast_possible_truncation)]
pub(crate) fn padded_unpadded_row_bytes(width: u32) -> (u32, u32) {
    let bytes_per_pixel = size_of::<u32>() as u32;
    let unpadded_bytes_per_row = width * bytes_per_pixel;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
    (
        unpadded_bytes_per_row + padded_bytes_per_row_padding,
        unpadded_bytes_per_row,
    )
}

pub(crate) fn create_instance() -> Instance {
    Instance::new(&wgpu::InstanceDescriptor {
        backends: Backends::from_env().unwrap_or_else(Backends::all),
        flags: InstanceFlags::default(),
        backend_options: BackendOptions::default(),
    })
}

pub(crate) fn create_adapter(instance: &Instance, window_surface: Option<&Surface<'_>>) -> Adapter {
    let adapter_request = RequestAdapterOptions {
        power_preference: PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: window_surface,
    };
    executor::block_on(instance.request_adapter(&adapter_request))
        .expect("no supported graphic adapter found")
}

pub(crate) fn create_device(adapter: &Adapter) -> (Device, Queue) {
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

pub(crate) fn create_buffer(device: &Device, label: &str, size: u64) -> Buffer {
    device.create_buffer(&BufferDescriptor {
        label: Some(label),
        size,
        usage: BufferUsages::STORAGE
            | BufferUsages::COPY_SRC
            | BufferUsages::UNIFORM
            | BufferUsages::VERTEX
            | BufferUsages::INDEX,
        mapped_at_creation: false,
    })
}

pub(crate) fn create_encoder(device: &Device) -> CommandEncoder {
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

pub(crate) fn create_target_texture(device: &Device, size: (u32, u32)) -> Texture {
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

pub(crate) fn create_depth_buffer(device: &Device, size: (u32, u32)) -> TextureView {
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

pub(crate) fn create_render_pass<'a>(
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

pub(crate) fn pipeline_layout(
    device: &Device,
    layout: Option<&BindGroupLayout>,
    label: &str,
) -> PipelineLayout {
    device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &layout.map_or(vec![], |layout| vec![layout]),
        push_constant_ranges: &[],
    })
}

// coverage: off (window cannot be tested)

pub(crate) fn create_window(event_loop: &ActiveEventLoop, size: (u32, u32)) -> Arc<Window> {
    let size = PhysicalSize::new(size.0, size.1);
    let window = event_loop
        .create_window(Window::default_attributes().with_inner_size(size))
        .expect("cannot create window");
    Arc::new(window)
}

pub(crate) fn create_surface(instance: &Instance, window: Arc<Window>) -> Surface<'static> {
    instance
        .create_surface(window)
        .expect("cannot create surface")
}

pub(crate) fn create_surface_config(
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

pub(crate) fn create_surface_view(texture: &SurfaceTexture) -> TextureView {
    texture
        .texture
        .create_view(&TextureViewDescriptor::default())
}

// coverage: on
