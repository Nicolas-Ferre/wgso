use crate::file::{File, Files};
use crate::type_::Type;
use crate::Error;
use fxhash::{FxHashMap, FxHashSet};
use itertools::Itertools;
use naga::back::wgsl::{Writer, WriterFlags};
use naga::valid::{Capabilities, ModuleInfo, ValidationFlags, Validator};
use std::iter;
use std::path::{Path, PathBuf};
use std::slice::Iter;
use std::sync::Arc;
use wgpu::naga;
use wgpu::naga::{AddressSpace, ResourceBinding};

pub(crate) const BINDING_GROUP: u32 = 0;

#[derive(Debug)]
pub(crate) struct Modules {
    modules: Vec<Arc<Module>>,
}

impl Modules {
    pub(crate) fn new(root_path: &Path, files: &Files, errors: &mut Vec<Error>) -> Self {
        Self {
            modules: files
                .iter()
                .filter_map(|file| match Module::new(root_path, file, files) {
                    Ok(module) => Some(Arc::new(module)),
                    Err(error) => {
                        errors.push(error);
                        None
                    }
                })
                .collect::<Vec<_>>(),
        }
    }

    pub(crate) fn iter(&self) -> Iter<'_, Arc<Module>> {
        self.modules.iter()
    }
}

#[derive(Debug)]
pub(crate) struct Module {
    pub(crate) files: Vec<Arc<File>>,
    pub(crate) bindings: FxHashMap<String, Binding>,
    pub(crate) code: String,
}

impl Module {
    pub(crate) fn new(root_path: &Path, file: &Arc<File>, files: &Files) -> Result<Self, Error> {
        let (code, files) = Self::extract_code(root_path, file, files)?;
        let mut parsed = naga::front::wgsl::parse_str(&code)
            .map_err(|error| Error::WgslParsing(files.clone(), error))?;
        Self::check_unsupported_features(&file.path, &parsed)?;
        let bindings = Self::configure_bindings(&mut parsed);
        Ok(Self {
            code: Self::write_code(&parsed, &files)?,
            files,
            bindings,
        })
    }

    pub(crate) fn binding_count(&self) -> usize {
        self.bindings.len()
    }

    pub(crate) fn storage_bindings(&self) -> impl Iterator<Item = (&String, &Binding)> + '_ {
        self.bindings
            .iter()
            .filter(|(_, binding)| binding.kind == BindingKind::Storage)
    }

    pub(crate) fn uniform_bindings(&self) -> impl Iterator<Item = (&String, &Binding)> + '_ {
        self.bindings
            .iter()
            .filter(|(_, binding)| binding.kind == BindingKind::Uniform)
    }

    pub(crate) fn uniform_names(&self) -> impl Iterator<Item = &String> + '_ {
        self.bindings
            .iter()
            .filter(|(_, binding)| binding.kind == BindingKind::Uniform)
            .map(|(name, _)| name)
    }

    pub(crate) fn uniform_binding(&self, name: &str) -> Option<&Binding> {
        self.bindings
            .iter()
            .find(|(binding_name, binding)| {
                binding.kind == BindingKind::Uniform && binding_name == &name
            })
            .map(|(_, binding)| binding)
    }

    fn write_code(parsed: &naga::Module, files: &[Arc<File>]) -> Result<String, Error> {
        let module_info = Self::validate_code(parsed, files)?;
        let mut code = String::new();
        Writer::new(&mut code, WriterFlags::empty())
            .write(parsed, &module_info)
            .expect("internal error: parsed WGSL code should be valid");
        Ok(code)
    }

    fn extract_code(
        root_path: &Path,
        file: &Arc<File>,
        files: &Files,
    ) -> Result<(String, Vec<Arc<File>>), Error> {
        let files: Vec<_> = Self::extract_file_paths(root_path, file, files)?
            .into_iter()
            .map(|path| files.get(&path).clone())
            .collect();
        let code = files
            .iter()
            .map(|file| {
                file.code
                    .lines()
                    .map(|line| {
                        if line.trim_start().starts_with('#') {
                            format!("{: ^1$}\n", "", line.len())
                        } else {
                            format!("{line}\n")
                        }
                    })
                    .join("")
            })
            .join("");
        Ok((code, files))
    }

    fn extract_file_paths(
        root_path: &Path,
        file: &Arc<File>,
        files: &Files,
    ) -> Result<Vec<PathBuf>, Error> {
        let mut paths: FxHashSet<_> = iter::once(file.path.clone()).collect();
        let mut last_path_count = 0;
        while last_path_count < paths.len() {
            last_path_count = paths.len();
            for path in paths.clone() {
                for directive in files.get(&path).directives.imports() {
                    let path = directive.file_path(root_path);
                    if files.exists(&path) {
                        paths.insert(path);
                    } else {
                        return Err(Error::DirectiveParsing(
                            directive.path[0].path.clone(),
                            directive.span.clone(),
                            format!("file at '{}' does not exist", path.display()),
                        ));
                    }
                }
            }
        }
        Ok(paths.into_iter().collect())
    }

    fn check_unsupported_features(path: &Path, parsed: &naga::Module) -> Result<(), Error> {
        if parsed.overrides.is_empty() {
            Ok(())
        } else {
            Err(Error::UnsupportedWgslFeature(
                path.to_path_buf(),
                "override constants are not supported by WGSO".to_string(),
            ))
        }
    }

    fn validate_code(parsed: &naga::Module, files: &[Arc<File>]) -> Result<ModuleInfo, Error> {
        match Validator::new(ValidationFlags::all(), Capabilities::all())
            .subgroup_stages(naga::valid::ShaderStages::all())
            .subgroup_operations(naga::valid::SubgroupOperationSet::all())
            .validate(parsed)
        {
            Ok(module_info) => Ok(module_info),
            Err(error) => Err(Error::WgslValidation(files.to_vec(), error)),
        }
    }

    fn configure_bindings(parsed: &mut naga::Module) -> FxHashMap<String, Binding> {
        let mut bindings: FxHashMap<_, _> = Self::configure_storage_bindings(parsed).collect();
        let storage_count = bindings.len();
        bindings.extend(Self::configure_uniform_bindings(parsed, storage_count));
        bindings
    }

    #[allow(clippy::cast_possible_truncation)]
    fn configure_storage_bindings(
        parsed: &mut naga::Module,
    ) -> impl Iterator<Item = (String, Binding)> + '_ {
        let types = &parsed.types;
        let parsed_clone = parsed.clone();
        parsed
            .global_variables
            .iter_mut()
            .filter(|(_, var)| matches!(var.space, AddressSpace::Storage { .. }))
            .enumerate()
            .filter_map(move |(index, (_, var))| {
                var.name.as_ref().map(|name| {
                    let binding_index = index as u32;
                    var.binding = Some(ResourceBinding {
                        group: BINDING_GROUP,
                        binding: binding_index,
                    });
                    (
                        name.clone(),
                        Binding {
                            kind: BindingKind::Storage,
                            type_: Arc::new(Type::new(&parsed_clone, &types[var.ty], 0)),
                            index: binding_index,
                        },
                    )
                })
            })
    }

    #[allow(clippy::cast_possible_truncation)]
    fn configure_uniform_bindings(
        parsed: &mut naga::Module,
        first_index: usize,
    ) -> impl Iterator<Item = (String, Binding)> + '_ {
        let types = &parsed.types;
        let parsed_clone = parsed.clone();
        parsed
            .global_variables
            .iter_mut()
            .filter(|(_, var)| matches!(var.space, AddressSpace::Uniform))
            .enumerate()
            .filter_map(move |(index, (_, var))| {
                var.name.as_ref().map(|name| {
                    let binding_index = (first_index + index) as u32;
                    var.binding = Some(ResourceBinding {
                        group: BINDING_GROUP,
                        binding: binding_index,
                    });
                    (
                        name.clone(),
                        Binding {
                            kind: BindingKind::Uniform,
                            type_: Arc::new(Type::new(&parsed_clone, &types[var.ty], 0)),
                            index: binding_index,
                        },
                    )
                })
            })
    }
}

#[derive(Debug)]
pub(crate) struct Binding {
    pub(crate) kind: BindingKind,
    pub(crate) type_: Arc<Type>,
    pub(crate) index: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BindingKind {
    Storage,
    Uniform,
}
