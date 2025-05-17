//! Math utils.

/// Normalize a `vec2f` value.
///
/// If the length of the vector is zero, then `vec2f(0, 0)` is returned.
fn normalize_vec2f_or_zero(value: vec2f) -> vec2f {
    return value / max(length(value), 1e-6);
}

/// Normalize a `vec3f` value.
///
/// If the length of the vector is zero, then `vec3f(0, 0, 0)` is returned.
fn normalize_vec3f_or_zero(value: vec3f) -> vec3f {
    return value / max(length(value), 1e-6);
}

/// Normalize a `vec4f` value.
///
/// If the length of the vector is zero, then `vec4f(0, 0, 0, 0)` is returned.
fn normalize_vec4f_or_zero(value: vec4f) -> vec4f {
    return value / max(length(value), 1e-6);
}
