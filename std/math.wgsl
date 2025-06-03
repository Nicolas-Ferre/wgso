/// Mathematical constants.
#mod constant

/// Archimedes’ constant (π).
const PI = 3.14159265358979323846264338327950288;

//! Vector utils.
#mod vector
#import ~.constant

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

/// Returns the angle in radians between two vectors.
///
/// Returned angle is between `0` and `2π`.
fn angle_vec2f(direction1: vec2f, direction2: vec2f) -> f32 {
    let angle = atan2(direction2.y, direction2.x) - atan2(direction1.y, direction1.x);
    return select(angle + 2 * PI, angle, angle >= 0.0);
}

/// Quaternion utilities.
#mod quaternion

/// The default quaternion (i.e. without any rotation).
const DEFAULT_QUAT = vec4f(0, 0, 0, 1);

/// Returns a quaterion to apply a rotation of `angle` radians around an `axis`.
fn quat(axis: vec3f, angle: f32) -> vec4f {
    let normalized_axis = normalize(axis);
    let half_angle = angle / 2;
    return vec4f(
        normalized_axis.x * sin(half_angle),
        normalized_axis.y * sin(half_angle),
        normalized_axis.z * sin(half_angle),
        cos(half_angle),
    );
}

/// Multiplies two quaternions.
///
/// This operation can be used to apply two consecutive rotations.
fn quat_mul(quat1: vec4f, quat2: vec4f) -> vec4f {
    return vec4f(
        quat1.w * quat2.x + quat1.x * quat2.w + quat1.y * quat2.z - quat1.z * quat2.y,
        quat1.w * quat2.y - quat1.x * quat2.z + quat1.y * quat2.w + quat1.z * quat2.x,
        quat1.w * quat2.z + quat1.x * quat2.y - quat1.y * quat2.x + quat1.z * quat2.w,
        quat1.w * quat2.w - quat1.x * quat2.x - quat1.y * quat2.y - quat1.z * quat2.z
    );
}

/// Inverts a quaternion.
///
/// This operation can be used to get the inverse rotation.
fn quat_inverse(quat: vec4f) -> vec4f {
    let squared_norm = dot(quat, quat);
    return vec4f(-quat.xyz, quat.w) / squared_norm;
}

/// Common matrices to apply transformations.
#mod matrix
#import ~.quaternion

/// Returns a projection matrix.
///
/// `ratio` is the ratio between screen width and height.
/// `fov` is in radians.
fn proj_mat(ratio: f32, fov: f32, far: f32, near: f32) -> mat4x4f {
    let focal_length = 1 / (2 * tan(fov / 2));
    return transpose(mat4x4f(
        focal_length, 0, 0, 0,
        0, focal_length * ratio, 0, 0,
        0, 0, far / (far - near), -far * near / (far - near),
        0, 0, 1, 0,
    ));
}

/// Returns a model transformation matrix.
fn model_mat(position: vec3f, scale: vec3f, rotation: vec4f) -> mat4x4f {
    return translation_mat(position) * rotation_mat(rotation) * scale_mat(scale);
}

/// Returns a view transformation matrix.
fn view_mat(position: vec3f, rotation: vec4f) -> mat4x4f {
    return rotation_mat(quat_inverse(rotation)) * translation_mat(-position);
}

/// Returns a translation matrix.
fn translation_mat(translation: vec3f) -> mat4x4f {
    return mat4x4f(
        1, 0, 0, 0,
        0, 1, 0, 0,
        0, 0, 1, 0,
        translation.x, translation.y, translation.z, 1,
    );
}

/// Returns a scaling matrix.
fn scale_mat(scale: vec3f) -> mat4x4f {
    return mat4x4f(
        scale.x, 0, 0, 0,
        0, scale.y, 0, 0,
        0, 0, scale.z, 0,
        0, 0, 0, 1,
    );
}

/// Returns a rotation matrix.
fn rotation_mat(quat: vec4f) -> mat4x4f {
    let x = quat.x;
    let y = quat.y;
    let z = quat.z;
    let w = quat.w;
    return mat4x4f(
        1 - 2 * y * y - 2 * z * z,  2 * x * y - 2 * z * w,      2 * x * z + 2 * y * w,      0,
        2 * x * y + 2 * z * w,      1 - 2 * x * x - 2 * z * z,  2 * y * z - 2 * x * w,      0,
        2 * x * z - 2 * y * w,      2 * y * z + 2 * x * w,      1 - 2 * x * x - 2 * y * y,  0,
        0,                          0,                          0,                          1,
    );
}

/// Random number generators.
#mod random

/// Generates a random `u32` value between `min` and `max`.
///
/// This function is based on [`random()`](random) function.
fn random_u32(seed: ptr<function, u32>, min: u32, max: u32) -> u32 {
    return random(seed) % max(abs(max - min), 1) + min(min, max);
}

/// Generates a random `i32` value between `min` and `max`.
///
/// This function is based on [`random()`](random) function.
fn random_i32(seed: ptr<function, u32>, min: i32, max: i32) -> i32 {
    return bitcast<i32>(random(seed)) % max(abs(max - min), 1) + min(min, max);
}

/// Generates a random `f32` value between `min` and `max`.
///
/// This function is based on [`random()`](random) function.
fn random_f32(seed: ptr<function, u32>, min: f32, max: f32) -> f32 {
    return f32(random(seed)) * abs(max - min) / f32(1 << 31) + min(min, max);
}

/// Generates a random `u32` value between 0 and 2^31 based on a `seed`.
///
/// The seed is modified in-place, and the new value can be used to generate another random value.
///
/// This function internally uses the
/// [LCG (Linear Congruential Genrator)](https://en.wikipedia.org/wiki/Linear_congruential_generator)
/// algorithm, which is fast but not cryptographically secure.
fn random(seed: ptr<function, u32>) -> u32 {
    *seed = (*seed * 1103515245 + 12345) % (1 << 31);
    return *seed;
}
