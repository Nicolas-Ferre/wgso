use crate::wgsl::Wgsl;
use crate::Error;
use regex::Regex;
use std::path::PathBuf;
use wgpu::naga::{AddressSpace, Span};

/// A parsed WGSL storage variable.
#[derive(Debug, Clone)]
pub struct Storage {
    pub(crate) name: String,
    pub(crate) size: u32,
    pub(crate) path: PathBuf,
    pub(crate) span: Span,
}

impl Storage {
    pub(crate) fn max_allowed_size() -> u32 {
        wgpu::Limits::default().max_storage_buffer_binding_size
    }

    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn extract(wgsl: &Wgsl, errors: &mut Vec<Error>) -> Vec<Self> {
        wgsl.module
            .global_variables
            .iter()
            .filter(|(_, var)| matches!(var.space, AddressSpace::Storage { .. }))
            .filter_map(|(_, var)| {
                var.name.as_ref().map(|name| {
                    // pattern should be valid because code has been successfully parsed with Naga
                    let var_pattern = Regex::new(&format!(r"> *({name}) *:"))
                        .expect("internal error: invalid storage pattern");
                    let var_pattern_match = var_pattern
                        .captures(&wgsl.code)
                        .expect("internal error: not found storage pattern")
                        .get(1)
                        .expect("internal error: not found storage pattern group");
                    let storage = Self {
                        name: name.clone(),
                        size: wgsl.module.types[var.ty].inner.size(wgsl.module.to_ctx()),
                        path: wgsl.path.clone(),
                        span: Span::new(
                            var_pattern_match.start() as u32,
                            var_pattern_match.end() as u32,
                        ),
                    };
                    if storage.size > Self::max_allowed_size() {
                        errors.push(Error::TooLargeStorage(storage));
                        None
                    } else {
                        Some(storage)
                    }
                })
            })
            .flatten()
            .collect()
    }
}
