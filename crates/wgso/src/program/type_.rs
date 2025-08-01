use crate::Error;
use naga::common::wgsl::{ToWgsl, TryToWgsl};
use naga::{AddressSpace, ArraySize, ImageClass, Scalar, ScalarKind, TypeInner, VectorSize};
use std::sync::Arc;
use wgso_parser::{ParsingError, Token};

#[derive(Debug, Eq, Clone)]
pub(crate) struct Type {
    pub(crate) size: u32,
    pub(crate) label: String,
    pub(crate) array_params: Option<(Box<Type>, u32)>,
    pub(crate) offset: u32, // relative to root parent type
    pub(crate) fields: Vec<TypeField>,
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        // comparing labels is not enough, as two types can have same name but different fields
        self.label == other.label && self.fields == other.fields
    }
}

impl Type {
    pub(crate) fn new(
        parsed_module: &naga::Module,
        parsed_type: &naga::Type,
        global_offset: u32,
    ) -> Self {
        Self {
            size: parsed_type.inner.size(parsed_module.to_ctx()),
            label: Self::label(parsed_module, parsed_type),
            offset: global_offset,
            array_params: if let TypeInner::Array { base, size, .. } = parsed_type.inner {
                Some((
                    Box::new(Self::new(parsed_module, &parsed_module.types[base], 0)),
                    Self::array_size_value(size).unwrap_or(1),
                ))
            } else {
                None
            },
            fields: Self::extract_fields(parsed_module, parsed_type, global_offset),
        }
    }

    pub(crate) fn field_ident_type(&self, fields: &[Token]) -> Result<&Self, Error> {
        if let Some((field_name, children)) = fields.split_first() {
            if let Some(field) = self
                .fields
                .iter()
                .find(|field| field.name == field_name.slice)
            {
                field.type_.field_ident_type(children)
            } else {
                Err(Error::DirectiveParsing(ParsingError {
                    path: field_name.path.clone(),
                    span: field_name.span.clone(),
                    message: format!("unknown field for type `{}`", self.label),
                }))
            }
        } else {
            Ok(self)
        }
    }

    pub(crate) fn field_name_type(&self, fields: &[&str]) -> Option<&Self> {
        if let Some((field_name, children)) = fields.split_first() {
            if let Some(field) = self.fields.iter().find(|field| &field.name == field_name) {
                field.type_.field_name_type(children)
            } else {
                None
            }
        } else {
            Some(self)
        }
    }

    pub(crate) fn is_vertex_compatible(&self) -> bool {
        self.label == "i32"
            || self.label == "u32"
            || self.label == "f32"
            || self.label.starts_with("vec2<")
            || self.label.starts_with("vec3<")
            || self.label.starts_with("vec4<")
    }

    fn label(parsed_module: &naga::Module, parsed_type: &naga::Type) -> String {
        if let Some(name) = &parsed_type.name {
            return name.clone();
        }
        match parsed_type.inner {
            TypeInner::Scalar(scalar) => Self::scalar_label(scalar),
            TypeInner::Vector { size, scalar } => format!(
                "vec{}<{}>",
                Self::vector_size_value(size),
                Self::scalar_label(scalar)
            ),
            TypeInner::Matrix {
                columns,
                rows,
                scalar,
            } => format!(
                "mat{}x{}<{}>",
                Self::vector_size_value(columns),
                Self::vector_size_value(rows),
                Self::scalar_label(scalar)
            ),
            TypeInner::Atomic(scalar) => format!("atomic<{}>", Self::scalar_label(scalar)),
            TypeInner::Pointer { base, space } => format!(
                "ptr<{}, {}>",
                Self::address_space_keyword(space),
                Self::label(parsed_module, &parsed_module.types[base])
            ),
            TypeInner::Array { base, size, .. } => format!(
                "array<{}{}>",
                Self::label(parsed_module, &parsed_module.types[base]),
                Self::array_size_param(size),
            ),
            TypeInner::BindingArray { size, base } => format!(
                "binding_array<{}{}>",
                Self::label(parsed_module, &parsed_module.types[base]),
                Self::array_size_param(size),
            ),
            TypeInner::Image {
                dim,
                arrayed,
                class,
            } => match class {
                ImageClass::Sampled { kind, multi } => format!(
                    "texture{}_{}{}<{}>",
                    if multi { "_multisampled" } else { "" },
                    dim.to_wgsl(),
                    if arrayed { "_array" } else { "" },
                    Self::scalar_kind_to_32bit_type(kind),
                ),
                ImageClass::Depth { multi } => format!(
                    "texture_depth{}_{}{}",
                    if multi { "_multisampled" } else { "" },
                    dim.to_wgsl(),
                    if arrayed { "_array" } else { "" },
                ),
                ImageClass::Storage { format, .. } => format!(
                    "texture_storage_{}{}<{}>",
                    dim.to_wgsl(),
                    if arrayed { "_array" } else { "" },
                    format.to_wgsl(),
                ),
            },
            TypeInner::Sampler { comparison } => {
                if comparison {
                    "sampler_comparison".into()
                } else {
                    "sampler".into()
                }
            }
            TypeInner::AccelerationStructure { .. } => {
                unreachable!("internal error: type should not be acceleration structure")
            }
            TypeInner::ValuePointer { .. } => {
                unreachable!("internal error: type should not be value pointer")
            }
            TypeInner::RayQuery { .. } => {
                unreachable!("internal error: type should not be ray query")
            }
            TypeInner::Struct { .. } => unreachable!("internal error: name should be present"),
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn extract_fields(
        parsed_module: &naga::Module,
        parsed_type: &naga::Type,
        global_offset: u32,
    ) -> Vec<TypeField> {
        match &parsed_type.inner {
            TypeInner::Struct { members, .. } => members
                .iter()
                .filter_map(|member| member.name.clone().map(|name| (name, member)))
                .map(|(name, member)| {
                    let parsed_member_type = &parsed_module.types[member.ty];
                    let member_type = Self::new(
                        parsed_module,
                        parsed_member_type,
                        global_offset + member.offset,
                    );
                    TypeField {
                        name,
                        type_: Arc::new(member_type),
                    }
                })
                .collect(),
            _ => vec![],
        }
    }

    fn scalar_label(scalar: Scalar) -> String {
        scalar.try_to_wgsl().unwrap_or("unknown").into()
    }

    fn vector_size_value(size: VectorSize) -> u32 {
        match size {
            VectorSize::Bi => 2,
            VectorSize::Tri => 3,
            VectorSize::Quad => 4,
        }
    }

    fn array_size_value(size: ArraySize) -> Option<u32> {
        match size {
            ArraySize::Constant(value) => Some(value.into()),
            ArraySize::Dynamic => None,
            ArraySize::Pending(_) => {
                unreachable!("internal error: WGSL override should not be accepted")
            }
        }
    }

    fn array_size_param(size: ArraySize) -> String {
        if let Some(value) = Self::array_size_value(size) {
            format!(", {value}")
        } else {
            String::new()
        }
    }

    fn address_space_keyword(space: AddressSpace) -> &'static str {
        match space {
            AddressSpace::Function => "function",
            AddressSpace::Private => "private",
            AddressSpace::WorkGroup => "workgroup",
            AddressSpace::Uniform => "uniform",
            AddressSpace::Storage { .. } => "storage",
            AddressSpace::Handle => {
                unreachable!("internal error: WGSL address space should not be handle")
            }
            AddressSpace::PushConstant => {
                unreachable!("internal error: WGSL address space should not be push constant")
            }
        }
    }

    fn scalar_kind_to_32bit_type(kind: ScalarKind) -> &'static str {
        match kind {
            ScalarKind::Sint => "i32",
            ScalarKind::Uint => "u32",
            ScalarKind::Float => "f32",
            ScalarKind::Bool => unreachable!("internal error: WGSL type should not be bool"),
            ScalarKind::AbstractInt | ScalarKind::AbstractFloat => {
                unreachable!("internal error: WGSL type should not be abstract")
            }
        }
    }
}

pub(crate) fn normalize_type_name(name: &str) -> String {
    let code = format!("var v: {name};");
    if let Ok(module) = naga::front::wgsl::parse_str(&code) {
        let (_, type_) = module
            .types
            .iter()
            .next()
            .expect("internal error: primitive type should have been parsed");
        Type::new(&module, type_, 0).label
    } else {
        name.into()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct TypeField {
    pub(crate) name: String,
    pub(crate) type_: Arc<Type>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::program::type_::Type;
    use naga::front::wgsl;

    #[test]
    fn parse_type_label() {
        assert_type_label("u32", None);
        assert_type_label("i32", None);
        assert_type_label("f32", None);
        assert_type_label("bool", None);
        assert_type_label("vec2<f32>", None);
        assert_type_label("vec3<f32>", None);
        assert_type_label("vec4<f32>", None);
        assert_type_label("mat4x2<f32>", None);
        assert_type_label("atomic<f32>", None);
        assert_type_label("ptr<function, f32>", None);
        assert_type_label("ptr<private, f32>", None);
        assert_type_label("ptr<workgroup, f32>", None);
        assert_type_label("ptr<uniform, f32>", None);
        assert_type_label("ptr<storage, f32>", None);
        assert_type_label("array<f32, 42>", None);
        assert_type_label("array<f32>", None);
        assert_type_label("MyStruct", None);
        assert_type_label("texture_2d<f32>", None);
        assert_type_label("texture_2d<i32>", None);
        assert_type_label("texture_2d<u32>", None);
        assert_type_label("texture_2d_array<f32>", None);
        assert_type_label("texture_cube<f32>", None);
        assert_type_label("texture_multisampled_2d<f32>", None);
        assert_type_label("texture_depth_multisampled_2d", None);
        assert_type_label(
            "texture_storage_2d<rgba8unorm, write>",
            Some("texture_storage_2d<rgba8unorm>"),
        );
        assert_type_label(
            "texture_storage_2d_array<rgba8unorm, write>",
            Some("texture_storage_2d_array<rgba8unorm>"),
        );
        assert_type_label("sampler", None);
        assert_type_label("sampler_comparison", None);
        assert_type_label("binding_array<f32, 4>", None);
    }

    fn assert_type_label(type_name: &str, expected_label: Option<&str>) {
        let code = format!("struct MyStruct {{field: f32}} var<storage> value: {type_name};");
        let module = wgsl::parse_str(&code).unwrap();
        let var = module.global_variables.iter().next().unwrap().1;
        let type_ = &module.types[var.ty];
        assert_eq!(
            Type::label(&module, type_),
            expected_label.unwrap_or(type_name)
        );
    }
}
