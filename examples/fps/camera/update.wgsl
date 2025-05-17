#shader<compute> update_camera

#import ~.storage
#import _.std.storage
#import _.std.matrix

const CAMERA_MOVEMENT_SPEED = 6;
const CAMERA_HORIZONTAL_ROTATION_SPEED = 400.;
const CAMERA_VERTICAL_ROTATION_SPEED = 200.;

@compute
@workgroup_size(1, 1, 1)
fn main() {
    camera.surface_ratio = f32(std_.surface.size.x) / f32(std_.surface.size.y);
    update_rotation_horizontal();
    update_rotation_vertical();
    update_position();
}

fn update_rotation_horizontal() {
    camera.horizontal_angle -= std_.mouse.delta.x / f32(std_.surface.size.x)
        * CAMERA_HORIZONTAL_ROTATION_SPEED * std_.time.frame_delta_secs;
    camera.rotation = quat(vec3f(0, 1, 0), camera.horizontal_angle);
}

fn update_rotation_vertical() {
    let direction = (rotation_mat(camera.rotation) * vec4f(1, 0, 0, 1)).xyz;
    camera.vertical_angle -= std_.mouse.delta.y / f32(std_.surface.size.y)
        * CAMERA_VERTICAL_ROTATION_SPEED * std_.time.frame_delta_secs;
    camera.vertical_angle = clamp(camera.vertical_angle, -PI / 2, PI / 2);
    camera.rotation = quat_mul(camera.rotation, quat(direction, camera.vertical_angle));
}

fn update_position() {
    let delta = input_direction(
        std_.keyboard.keys[KB_KEY_A],
        std_.keyboard.keys[KB_KEY_D],
        std_.keyboard.keys[KB_KEY_W],
        std_.keyboard.keys[KB_KEY_S],
    );
    let rotated_delta = rotation_mat(camera.rotation) * vec4f(delta.x, 0, delta.y, 1);
    camera.position += normalize_vec3f_or_zero(vec3f(rotated_delta.x, 0, rotated_delta.z))
        * CAMERA_MOVEMENT_SPEED * std_.time.frame_delta_secs;
}
