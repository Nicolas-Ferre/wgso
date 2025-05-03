#shader<render, Incompatible> vertex_type_incompatible

struct Incompatible {
    compatible_field: u32,
    incompatible_field: Inner,
}

struct Inner {
    field: u32
}
