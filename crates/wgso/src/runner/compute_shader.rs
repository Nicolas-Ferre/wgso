use crate::module::Module;
use wgpu::{
    BindGroupLayout, BindGroupLayoutEntry, BindingType, BufferBindingType, ComputePipeline,
    ComputePipelineDescriptor, Device, PipelineLayoutDescriptor, ShaderModuleDescriptor,
    ShaderStages,
};
use wgso_parser::Token;

#[derive(Debug)]
pub(crate) struct ComputeShaderResources {
    pub(crate) pipeline: ComputePipeline,
    pub(crate) layout: Option<BindGroupLayout>,
    pub(crate) directive: Vec<Token>,
}

impl ComputeShaderResources {
    pub(crate) fn new(directive: &[Token], module: &Module, device: &Device) -> Self {
        let layout = (module.binding_count() > 0)
            .then(|| Self::create_bind_group_layout(directive, module, device));
        let pipeline = Self::create_pipeline(module, directive, device, layout.as_ref());
        Self {
            pipeline,
            layout,
            directive: directive.to_vec(),
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn create_bind_group_layout(
        directive: &[Token],
        module: &Module,
        device: &Device,
    ) -> BindGroupLayout {
        let storage_entries = module
            .storage_bindings()
            .map(|(_, binding)| BindGroupLayoutEntry {
                binding: binding.index,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage {
                        read_only: binding.is_read_only,
                    },
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
            label: Some(&crate::directive::code(directive)),
            entries: &storage_entries.chain(uniform_entries).collect::<Vec<_>>(),
        })
    }

    fn create_pipeline(
        module: &Module,
        directive: &[Token],
        device: &Device,
        layout: Option<&BindGroupLayout>,
    ) -> ComputePipeline {
        let directive_code = crate::directive::code(directive);
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&directive_code),
            source: wgpu::ShaderSource::Wgsl(module.code.as_str().into()),
        });
        device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some(&directive_code),
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some(&directive_code),
                bind_group_layouts: &layout.map_or(vec![], |layout| vec![layout]),
                push_constant_ranges: &[],
            })),
            module: &module,
            entry_point: None,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
    }
}
