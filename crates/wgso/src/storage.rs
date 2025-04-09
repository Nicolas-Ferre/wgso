use crate::wgsl::Wgsl;
use naga::{AddressSpace, Span};
use regex::Regex;
use std::path::PathBuf;

/// A parsed WGSL storage variable.
#[derive(Debug, Clone)]
pub struct Storage {
    pub(crate) name: String,
    pub(crate) size: u32,
    pub(crate) path: PathBuf,
    pub(crate) span: Span,
}

impl Storage {
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn extract(wgsl: &Wgsl) -> impl Iterator<Item = Self> + '_ {
        wgsl.module
            .global_variables
            .iter()
            .filter(|(_, var)| matches!(var.space, AddressSpace::Storage { .. }))
            .filter_map(|(_, var)| {
                var.name.as_ref().map(|name| {
                    let var_pattern = Regex::new(&format!(r"> *({name}) *:"))
                        .expect("internal error: invalid storage pattern");
                    let var_pattern_match = var_pattern
                        .captures(&wgsl.code)
                        .expect("internal error: not found storage pattern")
                        .get(1)
                        .expect("internal error: not found storage pattern group");
                    Self {
                        name: name.clone(),
                        size: wgsl.module.types[var.ty].inner.size(wgsl.module.to_ctx()),
                        path: wgsl.path.clone(),
                        span: Span::new(
                            var_pattern_match.start() as u32,
                            var_pattern_match.end() as u32,
                        ),
                    }
                })
            })
    }
}
