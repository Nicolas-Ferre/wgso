use crate::directives::{Directive, DirectiveKind};
use crate::program::module::Module;
use crate::Program;
use fxhash::FxHashMap;
use wgpu::{BindGroup, BindGroupLayout, BindingResource, Buffer, BufferBinding, Device};

#[derive(Debug)]
pub(crate) struct ShaderExecution {
    pub(crate) shader_name: String,
    pub(crate) bind_group: Option<BindGroup>,
    pub(crate) is_init: bool,
    pub(crate) directive: Directive,
}

impl ShaderExecution {
    pub(crate) fn new(
        program: &Program,
        run_directive: &Directive,
        buffers: &FxHashMap<String, Buffer>,
        device: &Device,
        layout: Option<&BindGroupLayout>,
    ) -> Self {
        let directive_kind = run_directive.kind();
        let shader_name = run_directive.shader_name();
        let shader_module = if run_directive.kind() == DirectiveKind::Draw {
            &program.modules.render_shaders[&shader_name.slice].1
        } else {
            &program.modules.compute_shaders[&shader_name.slice].1
        };
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
            shader_name: shader_name.slice.clone(),
            bind_group,
            is_init: directive_kind == DirectiveKind::Init,
            directive: run_directive.clone(),
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn create_bind_group(
        program: &Program,
        run_directive: &Directive,
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
            let arg = run_directive.arg(name);
            let type_ = program.modules.storages[&arg.value.var.slice]
                .field_ident_type(&arg.value.fields)
                .expect("internal error: type field should be validated");
            wgpu::BindGroupEntry {
                binding: binding.index,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &buffers[&arg.value.var.slice],
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
            label: Some(&run_directive.code()),
            layout,
            entries: &storage_entries.chain(uniform_entries).collect::<Vec<_>>(),
        })
    }
}
