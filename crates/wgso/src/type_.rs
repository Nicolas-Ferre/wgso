use fxhash::FxHashMap;
use naga::common::wgsl::{ToWgsl, TryToWgsl};
use naga::{AddressSpace, ArraySize, ImageClass, Scalar, ScalarKind, TypeInner, VectorSize};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Type {
    pub(crate) size: u32,
    pub(crate) label: String,
    pub(crate) fields: FxHashMap<String, Type>, // used to compare two structs with same name
}

impl Type {
    pub(crate) fn new(parsed_module: &naga::Module, parsed_type: &naga::Type) -> Self {
        Self {
            size: parsed_type.inner.size(parsed_module.to_ctx()),
            label: Self::label(parsed_module, parsed_type),
            fields: Self::fields(parsed_module, parsed_type),
        }
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
                Self::array_size_value(size),
            ),
            TypeInner::BindingArray { size, base } => format!(
                "binding_array<{}{}>",
                Self::label(parsed_module, &parsed_module.types[base]),
                Self::array_size_value(size),
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
    fn fields(parsed_module: &naga::Module, parsed_type: &naga::Type) -> FxHashMap<String, Self> {
        match &parsed_type.inner {
            TypeInner::Struct { members, .. } => members
                .iter()
                .filter_map(|member| member.name.clone().map(|name| (name, member)))
                .map(|(name, member)| {
                    let type_ = Self::new(parsed_module, &parsed_module.types[member.ty]);
                    (name, type_)
                })
                .collect(),
            _ => FxHashMap::default(),
        }
    }

    fn scalar_label(scalar: Scalar) -> String {
        scalar
            .try_to_wgsl()
            .expect("internal error: unsupported WGSL type")
            .into()
    }

    fn vector_size_value(size: VectorSize) -> u32 {
        match size {
            VectorSize::Bi => 2,
            VectorSize::Tri => 3,
            VectorSize::Quad => 4,
        }
    }

    fn array_size_value(size: ArraySize) -> String {
        match size {
            ArraySize::Constant(value) => format!(", {value}"),
            ArraySize::Dynamic => String::new(),
            ArraySize::Pending(_) => {
                unreachable!("internal error: WGSL override should not be accepted")
            }
        }
    }

    fn address_space_keyword(space: AddressSpace) -> &'static str {
        match space {
            AddressSpace::Function => "function",
            AddressSpace::Private => "private",
            AddressSpace::WorkGroup => "workgroup",
            AddressSpace::Uniform => "uniform",
            AddressSpace::Storage { .. } => "storage",
            AddressSpace::Handle => "handle",
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
            ScalarKind::Bool => "bool",
            ScalarKind::AbstractInt | ScalarKind::AbstractFloat => {
                unreachable!("internal error: WGSL type should not be abstract")
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::type_::Type;
    use naga::front::wgsl;

    #[test]
    fn parse_type_label() {
        assert_type_label("u32", "u32");
        assert_type_label("i32", "i32");
        assert_type_label("f32", "f32");
        assert_type_label("bool", "bool");
        assert_type_label("vec2<f32>", "vec2<f32>");
        assert_type_label("vec3<f32>", "vec3<f32>");
        assert_type_label("vec4<f32>", "vec4<f32>");
        assert_type_label("mat4x2<f32>", "mat4x2<f32>");
        assert_type_label("atomic<f32>", "atomic<f32>");
        assert_type_label("ptr<storage, f32>", "ptr<storage, f32>");
        assert_type_label("array<f32, 42>", "array<f32, 42>");
        assert_type_label("array<f32>", "array<f32>");
        assert_type_label("MyStruct", "MyStruct");
        assert_type_label("texture_2d<f32>", "texture_2d<f32>");
        assert_type_label("texture_2d_array<f32>", "texture_2d_array<f32>");
        assert_type_label("texture_cube<f32>", "texture_cube<f32>");
        assert_type_label(
            "texture_multisampled_2d<f32>",
            "texture_multisampled_2d<f32>",
        );
        assert_type_label(
            "texture_depth_multisampled_2d",
            "texture_depth_multisampled_2d",
        );
        assert_type_label(
            "texture_storage_2d<rgba8unorm, write>",
            "texture_storage_2d<rgba8unorm>",
        );
        assert_type_label(
            "texture_storage_2d_array<rgba8unorm, write>",
            "texture_storage_2d_array<rgba8unorm>",
        );
        assert_type_label("sampler", "sampler");
        assert_type_label("sampler_comparison", "sampler_comparison");
        assert_type_label("binding_array<f32, 4>", "binding_array<f32, 4>");
    }

    fn assert_type_label(type_name: &str, expected_label: &str) {
        let code = format!("struct MyStruct {{field: f32}} var<storage> value: {type_name};");
        let module = wgsl::parse_str(&code).unwrap();
        let var = module.global_variables.iter().next().unwrap().1;
        let type_ = &module.types[var.ty];
        assert_eq!(Type::label(&module, type_), expected_label);
    }
}
