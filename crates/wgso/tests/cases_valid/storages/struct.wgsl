var<storage, read_write> struct_value: TestStruct;

struct TestStruct {
    vec3_field: vec3f,
    vec2_field: vec2f,
    inner_struct_field: InnerStruct,
}

struct InnerStruct {
    f32_field: f32,
}
