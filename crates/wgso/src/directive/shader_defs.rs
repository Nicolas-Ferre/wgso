use crate::directive::{Directive, DirectiveKind};
use crate::Error;
use wgso_parser::Token;

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
}

pub(crate) fn check(directives: &[Directive], errors: &mut Vec<Error>) {
    for (index, directive) in directives.iter().enumerate() {
        let kind = directive.kind();
        if DEF_DIRECTIVE_KINDS.contains(&kind) {
            check_duplicated(kind, directive, &directives[index + 1..], errors);
        }
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

fn shader_kind_name(kind: DirectiveKind) -> &'static str {
    match kind {
        DirectiveKind::ComputeShader => "compute",
        DirectiveKind::RenderShader => "render",
        DirectiveKind::Init | DirectiveKind::Run | DirectiveKind::Draw | DirectiveKind::Import => {
            unreachable!("internal error: unexpected directive kind")
        }
    }
}
