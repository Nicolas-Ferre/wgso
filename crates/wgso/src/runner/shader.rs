use crate::directive::run::RunDirective;
use crate::directive::shader::ShaderDirective;
use crate::module::Module;
use fxhash::FxHashMap;
use wgpu::{
    BindGroup, BindGroupLayout, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType,
    ComputePipeline, ComputePipelineDescriptor, Device, PipelineLayoutDescriptor,
    ShaderModuleDescriptor, ShaderStages,
};

#[derive(Debug)]
pub(crate) struct ComputeShaderResources {
    pub(crate) pipeline: ComputePipeline,
    pub(crate) layout: BindGroupLayout,
}

impl ComputeShaderResources {
    pub(crate) fn new(
        name: &str,
        directive: &ShaderDirective,
        module: &Module,
        device: &Device,
    ) -> Self {
        let layout = Self::create_bind_group_layout(directive, module, device);
        let pipeline = Self::create_pipeline(name, module, device, &layout);
        Self { pipeline, layout }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn create_bind_group_layout(
        directive: &ShaderDirective,
        module: &Module,
        device: &Device,
    ) -> BindGroupLayout {
        let storage_entries = module
            .storage_bindings()
            .map(|(_, binding)| BindGroupLayoutEntry {
                binding: binding.index,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
        let uniform_entries = module
            .uniform_bindings()
            .map(|(_, binding)| BindGroupLayoutEntry {
                binding: binding.index,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&directive.code),
            entries: &storage_entries.chain(uniform_entries).collect::<Vec<_>>(),
        })
    }

    fn create_pipeline(
        name: &str,
        module: &Module,
        device: &Device,
        layout: &BindGroupLayout,
    ) -> ComputePipeline {
        let label = format!("#shader<compute> {name}");
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&label),
            source: wgpu::ShaderSource::Wgsl(module.code.as_str().into()),
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
pub(crate) struct ComputeShaderRun {
    pub(crate) shader_name: String,
    pub(crate) bind_group: Option<BindGroup>,
}

impl ComputeShaderRun {
    pub(crate) fn new(
        run_directive: &RunDirective,
        shader_module: &Module,
        buffers: &FxHashMap<String, Buffer>,
        device: &Device,
        layout: &BindGroupLayout,
    ) -> Self {
        let bind_group = (shader_module.storage_bindings().count() > 0).then(|| {
            Self::create_bind_group(run_directive, shader_module, buffers, device, layout)
        });
        Self {
            shader_name: run_directive.name.label.clone(),
            bind_group,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn create_bind_group(
        run_directive: &RunDirective,
        shader_module: &Module,
        buffers: &FxHashMap<String, Buffer>,
        device: &Device,
        layout: &BindGroupLayout,
    ) -> BindGroup {
        let storage_entries =
            shader_module
                .storage_bindings()
                .map(|(name, binding)| wgpu::BindGroupEntry {
                    binding: binding.index,
                    resource: buffers[name].as_entire_binding(),
                });
        let uniform_entries =
            shader_module
                .uniform_bindings()
                .map(|(name, binding)| wgpu::BindGroupEntry {
                    binding: binding.index,
                    resource: buffers[&run_directive.args[name].value.label].as_entire_binding(),
                });
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&run_directive.code),
            layout,
            entries: &storage_entries.chain(uniform_entries).collect::<Vec<_>>(),
        })
    }
}
