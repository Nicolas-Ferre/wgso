use crate::directives::DirectiveKind;
use crate::program::file::File;
use crate::program::type_;
use crate::program::type_::Type;
use crate::Error;
use fxhash::FxHashMap;
use naga::back::wgsl::{Writer, WriterFlags};
use naga::valid::{Capabilities, ModuleInfo, ValidationFlags, Validator};
use naga::{AddressSpace, Module, ResourceBinding, StorageAccess, TypeInner};
use std::sync::Arc;

pub(crate) const BINDING_GROUP: u32 = 0;

#[derive(Debug)]
pub(crate) struct WgslModule {
    module: Module,
    pub(crate) files: Vec<Arc<File>>,
}

#[allow(clippy::cast_possible_truncation)]
impl WgslModule {
    pub(crate) fn new(code: &str, files: Vec<Arc<File>>) -> Result<Self, Error> {
        naga::front::wgsl::parse_str(code)
            .map_err(|error| Error::WgslParsing(files.clone(), error))
            .map(|module| Self { module, files })
            .and_then(Self::check_unsupported_features)
    }

    pub(crate) fn configure_bindings(&mut self) -> FxHashMap<String, Binding> {
        let mut bindings: FxHashMap<_, _> = self.configure_storage_bindings().collect();
        let storage_count = bindings.len();
        bindings.extend(self.configure_uniform_bindings(storage_count));
        bindings
    }

    pub(crate) fn configure_vertex_buffer(&mut self) {
        for file in &self.files {
            let render_shader_directives =
                crate::directives::find_all_by_kind(&file.directives, DirectiveKind::RenderShader);
            for directive in render_shader_directives {
                let vertex_type = directive.vertex_type();
                let vertex_type = type_::normalize_type_name(&vertex_type.slice);
                for (type_handle, type_) in self.module.types.clone().iter() {
                    if vertex_type != Type::new(&self.module, type_, 0).label {
                        continue;
                    }
                    let mut type_ = type_.clone();
                    let TypeInner::Struct { members, .. } = &mut type_.inner else {
                        continue;
                    };
                    for (index, member) in members.iter_mut().enumerate() {
                        let naga::Binding::Location { location, .. } =
                            member.binding.get_or_insert(naga::Binding::Location {
                                location: index as u32,
                                interpolation: None,
                                sampling: None,
                                blend_src: None,
                            })
                        else {
                            unreachable!("internal error: vertex location should be valid ")
                        };
                        *location = index as u32;
                    }
                    self.module.types.replace(type_handle, type_);
                    break;
                }
            }
        }
    }

    pub(crate) fn to_code(&self) -> Result<String, Error> {
        let module_info = self.validate_code()?;
        let mut code = String::new();
        Writer::new(&mut code, WriterFlags::empty())
            .write(&self.module, &module_info)
            .expect("internal error: parsed WGSL code should be valid");
        Ok(code)
    }

    pub(crate) fn extract_types(&self) -> FxHashMap<String, Type> {
        self.module
            .types
            .iter()
            .map(|(_, parsed_type)| {
                let type_ = Type::new(&self.module, parsed_type, 0);
                (type_.label.clone(), type_)
            })
            .collect()
    }

    fn check_unsupported_features(self) -> Result<Self, Error> {
        if self.module.overrides.is_empty() {
            Ok(self)
        } else {
            Err(Error::UnsupportedWgslFeature(
                self.files[0].path.clone(),
                "override constants are not supported by WGSO".to_string(),
            ))
        }
    }

    fn configure_storage_bindings(&mut self) -> impl Iterator<Item = (String, Binding)> + '_ {
        let types = &self.module.types;
        let parsed_clone = self.module.clone();
        self.module
            .global_variables
            .iter_mut()
            .filter_map(|(_, var)| {
                if let AddressSpace::Storage { access } = var.space {
                    Some((var, access))
                } else {
                    None
                }
            })
            .enumerate()
            .filter_map(move |(index, (var, access))| {
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
                            is_read_only: !access.contains(StorageAccess::STORE),
                        },
                    )
                })
            })
    }

    fn configure_uniform_bindings(
        &mut self,
        first_index: usize,
    ) -> impl Iterator<Item = (String, Binding)> + '_ {
        let types = &self.module.types;
        let parsed_clone = self.module.clone();
        self.module
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
                            is_read_only: true,
                        },
                    )
                })
            })
    }

    fn validate_code(&self) -> Result<ModuleInfo, Error> {
        match Validator::new(ValidationFlags::all(), Capabilities::all())
            .subgroup_stages(naga::valid::ShaderStages::all())
            .subgroup_operations(naga::valid::SubgroupOperationSet::all())
            .validate(&self.module)
        {
            Ok(module_info) => Ok(module_info),
            Err(error) => Err(Error::WgslValidation(self.files.clone(), error)),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Binding {
    pub(crate) kind: BindingKind,
    pub(crate) type_: Arc<Type>,
    pub(crate) index: u32,
    pub(crate) is_read_only: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BindingKind {
    Storage,
    Uniform,
}
