use crate::directive::Directive;
use crate::error::Error;
use crate::file::File;
use crate::{directive, wgsl_parsing};
use fxhash::FxHashMap;
use std::path::PathBuf;
use wgpu::naga::front::wgsl;
use wgpu::naga::{AddressSpace, Module};

#[derive(Debug, Clone)]
pub(crate) struct WgslModule {
    pub(crate) path: PathBuf,
    pub(crate) module: Module,
    pub(crate) directives: Vec<Directive>,
    pub(crate) storage_bindings: FxHashMap<String, Binding>,
    pub(crate) uniform_bindings: FxHashMap<String, Binding>,
    pub(crate) code: String,
    pub(crate) cleaned_code: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Binding {
    pub(crate) index: usize,
    pub(crate) type_: String,
}

impl WgslModule {
    pub(crate) fn parse(file: &File, errors: &mut Vec<Error>) -> Option<Self> {
        let (code_without_directives, directives) = Self::extract_directives(file, errors);
        match wgsl::parse_str(&code_without_directives) {
            Ok(module) => {
                let storage_bindings = module
                    .global_variables
                    .iter()
                    .filter(|(_, var)| matches!(var.space, AddressSpace::Storage { .. }))
                    .filter_map(|(_, var)| var.name.clone())
                    .enumerate()
                    .map(|(index, name)| {
                        let binding = Binding {
                            index,
                            type_: wgsl_parsing::storage_var_type(&code_without_directives, &name),
                        };
                        (name, binding)
                    })
                    .collect::<FxHashMap<_, _>>();
                let uniform_bindings = module
                    .global_variables
                    .iter()
                    .filter(|(_, var)| matches!(var.space, AddressSpace::Uniform))
                    .filter_map(|(_, var)| var.name.clone())
                    .enumerate()
                    .map(|(index, name)| {
                        let binding = Binding {
                            index: index + storage_bindings.len(),
                            type_: wgsl_parsing::uniform_var_type(&code_without_directives, &name),
                        };
                        (name, binding)
                    })
                    .collect::<FxHashMap<_, _>>();
                let cleaned_code = Self::add_bindings(
                    code_without_directives,
                    &storage_bindings,
                    &uniform_bindings,
                );
                Some(Self {
                    path: file.path.clone(),
                    storage_bindings,
                    uniform_bindings,
                    module,
                    directives,
                    code: file.code.clone(),
                    cleaned_code,
                })
            }
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

    fn add_bindings(
        mut code: String,
        storage_bindings: &FxHashMap<String, Binding>,
        uniform_bindings: &FxHashMap<String, Binding>,
    ) -> String {
        for (name, binding) in storage_bindings {
            let position = wgsl_parsing::storage_var_start(&code, name);
            code.insert_str(position, &format!("@group(0) @binding({}) ", binding.index));
        }
        for (name, binding) in uniform_bindings {
            let position = wgsl_parsing::uniform_var_start(&code, name);
            code.insert_str(position, &format!("@group(0) @binding({}) ", binding.index));
        }
        code
    }
}
