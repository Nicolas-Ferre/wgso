#mod main
#init ~.init()
#draw ~.render<cubes.vertices, light.point>(camera=camera)

const POINT_LIGHT_SIZE = vec3f(0.01, 0.01, 0.01);

struct Light {
    ambient: AmbientLight,
    point: PointLight,
}

struct PointLight {
    position: vec3f,
    color: vec3f,
}

struct AmbientLight {
    strength: f32,
}

#mod storage
#import ~.main

var<storage, read_write> light: Light;

#shader<compute> init
#import ~.storage
#import _.std.color.constant

@compute
@workgroup_size(1, 1, 1)
fn main() {
    light.ambient.strength = 0.05;
    light.point.position = vec3f(0.6, 0.6, -0.5);
    light.point.color = WHITE.rgb;
}

#shader<render, Vertex, PointLight> render
#import ~.main
#import camera.main
#import _.std.vertex.transform
#import _.std.vertex.type

var<uniform> camera: Camera;

struct Fragment {
    @builtin(position)
    clip_position: vec4f,
    @location(0)
    color: vec3f,
}

@vertex
fn vs_main(vertex: Vertex, instance: PointLight) -> Fragment {
    let projection = proj_mat(camera.surface_ratio, camera.fov, camera.far, camera.near);
    let view = view_mat(camera.position, vec3f(1, 1, 1), camera.rotation);
    let model = model_mat(instance.position, POINT_LIGHT_SIZE, DEFAULT_QUAT);
    let position = projection * view * model * vec4f(vertex.position, 1);
    return Fragment(position, instance.color);
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4f {
    return vec4f(fragment.color, 1);
}
