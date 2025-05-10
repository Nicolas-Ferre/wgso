#shader<compute> init_camera

#import ~.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    camera.position = vec3f(0, 1, -3);
    camera.rotation = quat(vec3f(1, 0, 0), -PI/16);
    camera.fov = radians(40);
    camera.far = 100;
    camera.near = 0.01;
}
