use crate::wgsl_module::WgslModule;
use crate::wgsl_parsing;
use std::ops::Range;
use std::path::PathBuf;
use wgpu::naga::AddressSpace;

/// A parsed WGSL storage variable.
#[derive(Debug, Clone)]
pub struct Storage {
    pub(crate) name: String,
    pub(crate) size: u32,
    pub(crate) path: PathBuf,
    pub(crate) span: Range<usize>,
    pub(crate) type_: String,
}

impl Storage {
    pub(crate) fn extract(wgsl: &WgslModule) -> Vec<Self> {
        wgsl.module
            .global_variables
            .iter()
            .filter(|(_, var)| matches!(var.space, AddressSpace::Storage { .. }))
            .filter_map(|(_, var)| {
                var.name.as_ref().map(|name| Self {
                    name: name.clone(),
                    size: wgsl.module.types[var.ty].inner.size(wgsl.module.to_ctx()),
                    path: wgsl.path.clone(),
                    span: wgsl_parsing::storage_name_span(&wgsl.code, name),
                    type_: wgsl.storage_bindings[name].type_.clone(),
                })
            })
            .collect()
    }
}
