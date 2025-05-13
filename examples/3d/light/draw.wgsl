#shader<render, Vertex, PointLight> light

#import ~.main
#import camera.main
#import _.std.matrix
#import _.std.vertex

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
    let view = view_mat(camera.position, camera.rotation);
    let model = model_mat(instance.position, POINT_LIGHT_SIZE, DEFAULT_QUAT);
    let position = projection * view * model * vec4f(vertex.position, 1);
    return Fragment(position, instance.color);
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4f {
    return vec4f(fragment.color, 1);
}
