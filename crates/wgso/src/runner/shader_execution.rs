use crate::directives::{Directive, DirectiveKind};
use crate::program::module::Module;
use crate::program::section::Section;
use crate::Program;
use fxhash::FxHashMap;
use std::path::PathBuf;
use wgpu::{BindGroup, BindGroupLayout, BindingResource, Buffer, BufferBinding, Device};

#[derive(Debug)]
pub(crate) struct ShaderExecution {
    pub(crate) shader_ident: (PathBuf, String),
    pub(crate) bind_group: Option<BindGroup>,
    pub(crate) is_init: bool,
    pub(crate) directive: Directive,
    pub(crate) toggle_var_names: Vec<String>,
    pub(crate) is_init_done: bool,
}

impl ShaderExecution {
    pub(crate) fn new(
        program: &Program,
        section: &Section,
        run_directive: &Directive,
        buffers: &FxHashMap<String, Option<Buffer>>,
        device: &Device,
        layout: Option<&BindGroupLayout>,
    ) -> Self {
        let directive_kind = run_directive.kind();
        let item_ident = run_directive.item_ident(&program.root_path);
        let mut execution = Self {
            shader_ident: item_ident,
            bind_group: None,
            is_init: directive_kind == DirectiveKind::Init,
            directive: run_directive.clone(),
            toggle_var_names: section.toggle_var_names.clone(),
            is_init_done: false,
        };
        execution.enable(program, buffers, device, layout);
        execution
    }

    pub(crate) fn disable(&mut self) {
        self.is_init_done = false;
        self.bind_group = None;
    }

    pub(crate) fn enable(
        &mut self,
        program: &Program,
        buffers: &FxHashMap<String, Option<Buffer>>,
        device: &Device,
        layout: Option<&BindGroupLayout>,
    ) {
        let item_ident = self.directive.item_ident(&program.root_path);
        let shader_module = if self.directive.kind() == DirectiveKind::Draw {
            &program.modules.render[&item_ident]
        } else {
            &program.modules.compute[&item_ident]
        };
        self.bind_group = layout
            .as_ref()
            .map(|layout| {
                Self::create_bind_group(
                    program,
                    &self.directive,
                    shader_module,
                    buffers,
                    device,
                    layout,
                )
            })
            .flatten();
    }

    #[allow(clippy::cast_possible_truncation)]
    fn create_bind_group(
        program: &Program,
        run_directive: &Directive,
        shader_module: &Module,
        buffers: &FxHashMap<String, Option<Buffer>>,
        device: &Device,
        layout: &BindGroupLayout,
    ) -> Option<BindGroup> {
        let storage_entries = shader_module
            .storage_bindings()
            .map(|(name, binding)| {
                Some(wgpu::BindGroupEntry {
                    binding: binding.index,
                    resource: buffers[name].as_ref()?.as_entire_binding(),
                })
            })
            .collect::<Option<Vec<_>>>()?;
        let uniform_entries = shader_module
            .uniform_bindings()
            .map(|(name, binding)| {
                let arg = run_directive.arg(name);
                let type_ = program.modules.storages[&arg.value.var.slice]
                    .type_
                    .field_ident_type(&arg.value.fields)
                    .expect("internal error: type field should be validated");
                Some(wgpu::BindGroupEntry {
                    binding: binding.index,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: buffers[&arg.value.var.slice].as_ref()?,
                        offset: type_.offset.into(),
                        size: Some(
                            u64::from(type_.size)
                                .try_into()
                                .expect("internal error: type size should be validated"),
                        ),
                    }),
                })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&run_directive.code()),
                layout,
                entries: &storage_entries
                    .into_iter()
                    .chain(uniform_entries)
                    .collect::<Vec<_>>(),
            }),
        )
    }
}
