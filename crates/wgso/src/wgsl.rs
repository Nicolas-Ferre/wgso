use crate::directive;
use crate::directive::Directive;
use crate::error::Error;
use crate::file::File;
use std::path::PathBuf;
use wgpu::naga::front::wgsl;
use wgpu::naga::{AddressSpace, Module};

#[derive(Debug, Clone)]
pub(crate) struct WgslModule {
    pub(crate) path: PathBuf,
    pub(crate) module: Module,
    pub(crate) directives: Vec<Directive>,
    pub(crate) storages: Vec<String>,
    pub(crate) code: String,
    pub(crate) cleaned_code: String,
}

impl WgslModule {
    pub(crate) fn parse(file: &File, errors: &mut Vec<Error>) -> Option<Self> {
        let (cleaned_code, directives) = Self::extract_directives(file, errors);
        match wgsl::parse_str(&cleaned_code) {
            Ok(module) => Some(Self {
                path: file.path.clone(),
                storages: module
                    .global_variables
                    .iter()
                    .filter(|(_, var)| matches!(var.space, AddressSpace::Storage { .. }))
                    .filter_map(|(_, var)| var.name.clone())
                    .collect(),
                module,
                directives,
                code: file.code.clone(),
                cleaned_code,
            }),
            Err(error) => {
                errors.push(Error::WgslParsing(file.path.clone(), error));
                None
            }
        }
    }

    fn extract_directives(file: &File, errors: &mut Vec<Error>) -> (String, Vec<Directive>) {
        let mut cleaned_code = String::new();
        let mut directives = vec![];
        for line in file.code.lines() {
            if let Some(directive) = line.trim_start().strip_prefix('#') {
                match directive::parse(
                    directive,
                    &file.path,
                    cleaned_code.len() + line.len() - directive.len(),
                ) {
                    Ok(directive) => directives.push(directive),
                    Err(error) => errors.push(error),
                }
                for _ in 0..line.len() {
                    cleaned_code.push(' ');
                }
            } else {
                cleaned_code += line;
            }
            cleaned_code.push('\n');
        }
        (cleaned_code, directives)
    }
}
