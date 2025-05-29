#init ~.init.init_camera()
#run ~.update.update_camera()

#import _.std.quaternion

struct Camera {
    position: vec3f,
    rotation: vec4f,
    fov: f32,
    far: f32,
    near: f32,
    surface_ratio: f32,
}
