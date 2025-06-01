use crate::directives::{Directive, DirectiveKind};
use crate::program::file::Files;
use crate::program::module::{Module, Modules};
use crate::Error;
use std::sync::Arc;
use wgso_parser::{ParsingError, Token};

const DEF_DIRECTIVE_KINDS: &[DirectiveKind] = &[
    DirectiveKind::Mod,
    DirectiveKind::ComputeShader,
    DirectiveKind::RenderShader,
];

impl Directive {
    pub(crate) fn workgroup_count(&self) -> (u16, u16, u16) {
        assert_eq!(self.kind(), DirectiveKind::ComputeShader);
        let mut tokens = self.find_all_by_label("workgroup_count");
        let workgroup_count_x = tokens.next().map_or(1, Self::convert_to_integer);
        let workgroup_count_y = tokens.next().map_or(1, Self::convert_to_integer);
        let workgroup_count_z = tokens.next().map_or(1, Self::convert_to_integer);
        (workgroup_count_x, workgroup_count_y, workgroup_count_z)
    }

    pub(crate) fn vertex_type(&self) -> &Token {
        assert_eq!(self.kind(), DirectiveKind::RenderShader);
        self.find_one_by_label("vertex_type")
    }

    pub(crate) fn instance_type(&self) -> &Token {
        assert_eq!(self.kind(), DirectiveKind::RenderShader);
        self.find_one_by_label("instance_type")
    }
}

pub(crate) fn check(files: &Files, errors: &mut Vec<Error>) {
    for (index, directive) in files.directives.iter().enumerate() {
        if DEF_DIRECTIVE_KINDS.contains(&directive.kind()) {
            check_duplicated(directive, &files.directives[index..], errors);
        }
    }
}

pub(crate) fn check_params(modules: &Modules, errors: &mut Vec<Error>) {
    for module in modules.render.values() {
        let directive = module.main_directive();
        check_buffer_type(directive.vertex_type(), module, errors);
        check_buffer_type(directive.instance_type(), module, errors);
    }
}

fn check_duplicated(directive: &Directive, all_directives: &[Directive], errors: &mut Vec<Error>) {
    let path = directive.path();
    let name = directive.section_name();
    errors.extend(
        all_directives
            .iter()
            .find(|other_directive| {
                DEF_DIRECTIVE_KINDS.contains(&other_directive.kind())
                    && other_directive.path() == path
                    && other_directive.section_name().slice == name.slice
                    && other_directive.span() != directive.span()
            })
            .map(|directive| Error::ModuleConflict(name.clone(), directive.section_name().clone())),
    );
}

fn check_buffer_type(type_: &Token, module: &Arc<Module>, errors: &mut Vec<Error>) {
    if let Some(vertex_type) = module.type_(&type_.slice) {
        for field in &vertex_type.fields {
            if !field.type_.is_vertex_compatible() {
                errors.push(Error::DirectiveParsing(ParsingError {
                    path: type_.path.clone(),
                    span: type_.span.clone(),
                    message: format!(
                        "field `{}` of type `{}` cannot be used as vertex data",
                        field.name, field.type_.label
                    ),
                }));
            }
        }
    } else {
        errors.push(Error::DirectiveParsing(ParsingError {
            path: type_.path.clone(),
            span: type_.span.clone(),
            message: format!("type `{}` not found", type_.slice),
        }));
    }
}
