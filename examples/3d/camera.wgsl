#mod main
#init ~.init()
#run ~.update()
#import _.std.math.quaternion

struct Camera {
    position: vec3f,
    rotation: vec4f,
    fov: f32,
    far: f32,
    near: f32,
    surface_ratio: f32,
}


#mod storage
#import ~.main

var<storage, read_write> camera: Camera;

#shader<compute> init
#import ~.storage
#import _.std.math.constant

@compute
@workgroup_size(1, 1, 1)
fn main() {
    camera.position = vec3f(0, 1, -3);
    camera.rotation = quat(vec3f(1, 0, 0), -PI/16);
    camera.fov = radians(40);
    camera.far = 100;
    camera.near = 0.01;
}

#shader<compute> update
#import ~.storage
#import _.std.state.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    camera.surface_ratio = f32(std_.surface.size.x) / f32(std_.surface.size.y);
}
