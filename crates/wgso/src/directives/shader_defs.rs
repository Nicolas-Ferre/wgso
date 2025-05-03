use crate::directives::{Directive, DirectiveKind};
use crate::program::module::{Module, Modules};
use crate::Error;
use std::sync::Arc;
use wgso_parser::{ParsingError, Token};

const DEF_DIRECTIVE_KINDS: &[DirectiveKind] =
    &[DirectiveKind::ComputeShader, DirectiveKind::RenderShader];

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

pub(crate) fn check(directives: &[Directive], errors: &mut Vec<Error>) {
    for (index, directive) in directives.iter().enumerate() {
        let kind = directive.kind();
        if DEF_DIRECTIVE_KINDS.contains(&kind) {
            check_duplicated(kind, directive, &directives[index + 1..], errors);
        }
    }
}

pub(crate) fn check_params(modules: &Modules, errors: &mut Vec<Error>) {
    for (directive, module) in modules.render_shaders.values() {
        check_buffer_type(directive.vertex_type(), module, errors);
        check_buffer_type(directive.instance_type(), module, errors);
    }
}

fn check_duplicated(
    kind: DirectiveKind,
    directive: &Directive,
    other_directives: &[Directive],
    errors: &mut Vec<Error>,
) {
    let shader_name = directive.shader_name();
    errors.extend(
        other_directives
            .iter()
            .find(|directive| {
                directive.kind() == kind && directive.shader_name().slice == shader_name.slice
            })
            .map(|directive| {
                Error::ShaderConflict(
                    shader_name.clone(),
                    directive.shader_name().clone(),
                    shader_kind_name(kind),
                )
            }),
    );
}

fn check_buffer_type(type_: &Token, module: &Arc<Module>, errors: &mut Vec<Error>) {
    if let Some(vertex_type) = module.type_(&type_.slice) {
        for (name, field_type) in &vertex_type.fields {
            if !field_type.is_vertex_compatible() {
                errors.push(Error::DirectiveParsing(ParsingError {
                    path: type_.path.clone(),
                    span: type_.span.clone(),
                    message: format!(
                        "field `{name}` of type `{}` cannot be used as vertex data",
                        field_type.label
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

fn shader_kind_name(kind: DirectiveKind) -> &'static str {
    match kind {
        DirectiveKind::ComputeShader => "compute",
        DirectiveKind::RenderShader => "render",
        DirectiveKind::Init | DirectiveKind::Run | DirectiveKind::Draw | DirectiveKind::Import => {
            unreachable!("internal error: unexpected directive kind")
        }
    }
}
