#mod main

struct Camera {
    position: vec3f,
    horizontal_angle: f32,
    vertical_angle: f32,
    rotation: vec4f,
    fov: f32,
    far: f32,
    near: f32,
}

#mod compute
#import ~.main
#import _.std.math.constant
#import _.std.math.quaternion
#import _.std.math.matrix
#import _.std.io.compute
#import _.std.input.keyboard

const CAMERA_MOVEMENT_SPEED = 6;
const CAMERA_HORIZONTAL_ROTATION_SPEED = 400.;
const CAMERA_VERTICAL_ROTATION_SPEED = 200.;
const ANGLE_EPSILON = 0.01;

fn init_camera() -> Camera {
    let position = vec3f(0, 2, -10);
    let vertical_angle = 0.;
    let horizontal_angle = 0.;
    let rotation = DEFAULT_QUAT;
    let fov = radians(60);
    let far = 100.;
    let near = 0.01;
    return Camera(position, vertical_angle, horizontal_angle, rotation, fov, far, near);
}

fn update_camera(camera: Camera) -> Camera {
    var updated = camera;
    updated = _update_camera_horizontal_rotation(updated);
    updated = _update_camera_vertical_rotation(updated);
    updated = _update_camera_position(updated);
    return updated;
}

fn _update_camera_horizontal_rotation(camera: Camera) -> Camera {
    var updated = camera;
    let speed = CAMERA_HORIZONTAL_ROTATION_SPEED * std_.time.frame_delta_secs;
    updated.horizontal_angle -= std_.mouse.delta.x / f32(std_.surface.size.x) * speed;
    updated.rotation = quat(vec3f(0, 1, 0), camera.horizontal_angle);
    return updated;
}

fn _update_camera_vertical_rotation(camera: Camera) -> Camera {
    var updated = camera;
    let direction = (rotation_mat(camera.rotation) * vec4f(1, 0, 0, 1)).xyz;
    let speed = CAMERA_VERTICAL_ROTATION_SPEED * std_.time.frame_delta_secs;
    updated.vertical_angle -= std_.mouse.delta.y / f32(std_.surface.size.y) * speed;
    updated.vertical_angle = clamp(updated.vertical_angle, -PI / 2 + ANGLE_EPSILON, PI / 2 - ANGLE_EPSILON);
    updated.rotation = quat_mul(updated.rotation, quat(direction, updated.vertical_angle));
    return updated;
}

fn _update_camera_position(camera: Camera) -> Camera {
    var updated = camera;
    let delta = input_direction(
        std_.keyboard.keys[KB_KEY_A],
        std_.keyboard.keys[KB_KEY_D],
        std_.keyboard.keys[KB_KEY_W],
        std_.keyboard.keys[KB_KEY_S],
    );
    let rotated_delta = rotation_mat(updated.rotation) * vec4f(delta.x, 0, delta.y, 1);
    let speed = CAMERA_MOVEMENT_SPEED * std_.time.frame_delta_secs;
    updated.position += normalize_vec3f_or_zero(vec3f(rotated_delta.x, 0, rotated_delta.z)) * speed;
    return updated;
}
