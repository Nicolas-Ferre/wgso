/// Color constants.
#mod constant

/// <span style="color:black">█</span>
const BLACK = vec4f(0., 0., 0., 1.);
/// <span style="color:#404040">█</span>
const DARK_GRAY = vec4f(0.25, 0.25, 0.25, 1.);
/// <span style="color:gray">█</span>
const GRAY = vec4f(0.5, 0.5, 0.5, 1.);
/// <span style="color:silver">█</span>
const SILVER = vec4f(0.75, 0.75, 0.75, 1.);
/// <span style="color:white">█</span>
const WHITE = vec4f(1., 1., 1., 1.);
/// <span style="color:red">█</span>
const RED = vec4f(1., 0., 0., 1.);
/// <span style="color:green">█</span>
const GREEN = vec4f(0., 1., 0., 1.);
/// <span style="color:blue">█</span>
const BLUE = vec4f(0., 0., 1., 1.);
/// <span style="color:yellow">█</span>
const YELLOW = vec4f(1., 1., 0., 1.);
/// <span style="color:cyan">█</span>
const CYAN = vec4f(0., 1., 1., 1.);
/// <span style="color:magenta">█</span>
const MAGENTA = vec4f(1., 0., 1., 1.);
/// <span style="color:maroon">█</span>
const MAROON = vec4f(0.5, 0., 0., 1.);
/// <span style="color:#006400">█</span>
const DARK_GREEN = vec4f(0., 0.5, 0., 1.);
/// <span style="color:navy">█</span>
const NAVY = vec4f(0., 0., 0.5, 1.);
/// <span style="color:olive">█</span>
const OLIVE = vec4f(0.5, 0.5, 0., 1.);
/// <span style="color:teal">█</span>
const TEAL = vec4f(0., 0.5, 0.5, 1.);
/// <span style="color:purple">█</span>
const PURPLE = vec4f(0.5, 0., 0.5, 1.);
/// No color
const INVISIBLE = vec4f(0., 0., 0., 0.);


/// Utilities to apply lighting in a scene.
#mod lighting

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
