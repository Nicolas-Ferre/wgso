//! Common matrices to apply transformations.

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
