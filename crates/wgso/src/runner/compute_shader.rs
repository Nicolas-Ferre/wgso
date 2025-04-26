use crate::directive::compute_shader::ComputeShaderDirective;
use crate::directive::run::RunDirective;
use crate::module::Module;
use crate::Program;
use fxhash::FxHashMap;
use wgpu::{
    BindGroup, BindGroupLayout, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
    BufferBinding, BufferBindingType, ComputePipeline, ComputePipelineDescriptor, Device,
    PipelineLayoutDescriptor, ShaderModuleDescriptor, ShaderStages,
};

#[derive(Debug)]
pub(crate) struct ComputeShaderResources {
    pub(crate) pipeline: ComputePipeline,
    pub(crate) layout: Option<BindGroupLayout>,
    pub(crate) directive: ComputeShaderDirective,
}

impl ComputeShaderResources {
    pub(crate) fn new(
        directive: &ComputeShaderDirective,
        module: &Module,
        device: &Device,
    ) -> Self {
        let layout = (module.binding_count() > 0)
            .then(|| Self::create_bind_group_layout(directive, module, device));
        let pipeline = Self::create_pipeline(module, directive, device, layout.as_ref());
        Self {
            pipeline,
            layout,
            directive: directive.clone(),
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn create_bind_group_layout(
        directive: &ComputeShaderDirective,
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
            label: Some(&directive.code),
            entries: &storage_entries.chain(uniform_entries).collect::<Vec<_>>(),
        })
    }

    fn create_pipeline(
        module: &Module,
        directive: &ComputeShaderDirective,
        device: &Device,
        layout: Option<&BindGroupLayout>,
    ) -> ComputePipeline {
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&directive.code),
            source: wgpu::ShaderSource::Wgsl(module.code.as_str().into()),
        });
        device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some(&directive.code),
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some(&directive.code),
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

#[derive(Debug)]
pub(crate) struct ComputeShaderRun {
    pub(crate) shader_name: String,
    pub(crate) bind_group: Option<BindGroup>,
    pub(crate) is_init: bool,
}

impl ComputeShaderRun {
    pub(crate) fn new(
        program: &Program,
        run_directive: &RunDirective,
        buffers: &FxHashMap<String, Buffer>,
        device: &Device,
        layout: Option<&BindGroupLayout>,
    ) -> Self {
        let shader_module = &program.resources.compute_shaders[&run_directive.shader_name.label].1;
        let bind_group = layout.as_ref().map(|layout| {
            Self::create_bind_group(
                program,
                run_directive,
                shader_module,
                buffers,
                device,
                layout,
            )
        });
        Self {
            shader_name: run_directive.shader_name.label.clone(),
            bind_group,
            is_init: run_directive.is_init,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn create_bind_group(
        program: &Program,
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
        let uniform_entries = shader_module.uniform_bindings().map(|(name, binding)| {
            let type_ = program.resources.storages
                [&run_directive.args[name].value.buffer_name.label]
                .field_ident_type(&run_directive.args[name].value.fields)
                .expect("internal error: type field should be validated");
            wgpu::BindGroupEntry {
                binding: binding.index,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &buffers[&run_directive.args[name].value.buffer_name.label],
                    offset: type_.offset.into(),
                    size: Some(
                        u64::from(type_.size)
                            .try_into()
                            .expect("internal error: type size should be validated"),
                    ),
                }),
            }
        });
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&run_directive.code),
            layout,
            entries: &storage_entries.chain(uniform_entries).collect::<Vec<_>>(),
        })
    }
}
