//! Utilities to apply lighting in a scene.

/// Calculates the diffuse part of the Blinn–Phong reflection model.
fn diffuse_strength(
    frag_position: vec3f,
    frag_normal: vec3f,
    light_position: vec3f,
) -> f32 {
    let light_dir = normalize(light_position - frag_position);
    return max(dot(normalize(frag_normal), light_dir), 0.0);
}

/// Calculates the specular part of the Blinn–Phong reflection model.
fn specular_strength(
    frag_position: vec3f,
    frag_normal: vec3f,
    light_position: vec3f,
    view_position: vec3f,
    hardness: f32,
) -> f32 {
    let light_dir = normalize(light_position - frag_position);
    let view_dir = normalize(view_position - frag_position);
    let half_dir = normalize(view_dir + light_dir);
    return pow(max(dot(normalize(frag_normal), half_dir), 0.0), hardness);
}
