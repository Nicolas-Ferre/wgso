#mod main

struct Camera {
    position: vec3f,
    rotation: vec4f,
    fov: f32,
    far: f32,
    near: f32,
}

#mod compute
#import ~.main
#import _.std.math.constant
#import _.std.math.quaternion

fn init_camera() -> Camera {
    let position = vec3f(0, 1, -3);
    let rotation = quat(vec3f(1, 0, 0), -PI / 16);
    let fov = radians(40);
    let far = 100.;
    let near = 0.01;
    return Camera(position, rotation, fov, far, near);
}
