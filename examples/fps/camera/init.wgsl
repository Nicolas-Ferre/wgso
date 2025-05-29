#mod<compute> init_camera

#import ~.storage
#import _.std.quaternion

@compute
@workgroup_size(1, 1, 1)
fn main() {
    camera.position = vec3f(0, 2, -10);
    camera.vertical_angle = 0;
    camera.horizontal_angle = 0;
    camera.rotation = DEFAULT_QUAT;
    camera.fov = radians(60);
    camera.far = 100;
    camera.near = 0.01;
}
