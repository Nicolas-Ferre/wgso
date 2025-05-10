//! Quaternion utilities.

#import ~.constants

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
