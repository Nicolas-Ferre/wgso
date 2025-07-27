use crate::directives::defs::DEF_DIRECTIVE_KINDS;
use crate::directives::{BufferRef, Directive, DirectiveKind};
use crate::program::module::Modules;
use crate::program::section::{Section, Sections};
use crate::Error;
use std::path::Path;
use wgso_parser::ParsingError;

impl Directive {
    pub(crate) fn toggle_value_buffer(&self) -> BufferRef {
        self.buffer("toggle_value_buffer_var", "toggle_value_buffer_field")
    }
}

pub(crate) fn check(
    sections: &Sections,
    modules: &Modules,
    root_path: &Path,
    errors: &mut Vec<Error>,
) {
    for toggle_directive in sections.toggle_directives() {
        let path_prefix = toggle_directive.segment_path(root_path);
        let mut is_path_found = false;
        for section in sections.iter() {
            if has_section_path_prefix(&section.directive, &path_prefix) {
                is_path_found = true;
            } else {
                check_external_section(section, modules, &path_prefix, root_path, errors);
            }
        }
        if !is_path_found {
            errors.push(Error::DirectiveParsing(ParsingError {
                path: toggle_directive.path().into(),
                span: toggle_directive.item_span(),
                message: "no module matching toggled prefix".into(),
            }));
        }
        let toggle_value_buffer = toggle_directive.toggle_value_buffer();
        if let Some(buffer_type) = super::find_buffer_type(&toggle_value_buffer, modules, errors) {
            if buffer_type.label != "u32" {
                errors.push(Error::DirectiveParsing(ParsingError {
                    path: toggle_directive.path().into(),
                    span: toggle_value_buffer.span,
                    message: format!(
                        "found toggle value with type `{}`, expected type `u32`",
                        buffer_type.label
                    ),
                }));
            }
        }
    }
}

fn check_external_section(
    section: &Section,
    module: &Modules,
    toggleable_path_prefix: &Path,
    root_path: &Path,
    errors: &mut Vec<Error>,
) {
    let import_directives = section
        .directives()
        .filter(|directive| directive.kind() == DirectiveKind::Import);
    for import_directive in import_directives {
        if import_directive
            .segment_path(root_path)
            .starts_with(toggleable_path_prefix)
        {
            errors.push(Error::DirectiveParsing(ParsingError {
                path: import_directive.path().into(),
                span: import_directive.span(),
                message: "cannot import a toggleable module from outside".into(),
            }));
        }
    }
    for directive in section.directives() {
        for buffer_ref in directive.buffers() {
            let storage = &module.storages[&buffer_ref.var.slice];
            let is_declared_in_toggleable_module = storage
                .declarations
                .iter()
                .any(|decl| decl.raw_module_path.starts_with(toggleable_path_prefix));
            if is_declared_in_toggleable_module && !storage.is_declared_in_non_toggleable_module {
                errors.push(Error::DirectiveParsing(ParsingError {
                    path: buffer_ref.var.path,
                    span: buffer_ref.var.span,
                    message: "buffer storage defined in a toggleable module cannot be used outside this module".into(),
                }));
            }
        }
    }
}

pub(crate) fn has_section_path_prefix(directive: &Directive, prefix: &Path) -> bool {
    assert!(DEF_DIRECTIVE_KINDS.contains(&directive.kind()));
    prefix.with_extension("wgsl") == directive.path()
        || directive
            .path()
            .with_extension("")
            .join(&directive.section_name().slice)
            .starts_with(prefix)
}
