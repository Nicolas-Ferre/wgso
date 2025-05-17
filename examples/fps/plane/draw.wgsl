#shader<render, Vertex, PlaneInstance> plane

#import ~.main
#import camera.main
#import _.std.matrix
#import _.std.vertex

const PLANE_POSITION = vec3f(0, 0, 0);

var<uniform> camera: Camera;

struct Fragment {
    @builtin(position)
    clip_position: vec4f,
    @location(0)
    color: vec4f,
}

@vertex
fn vs_main(vertex: Vertex, instance: PlaneInstance) -> Fragment {
    let projection = proj_mat(camera.surface_ratio, camera.fov, camera.far, camera.near);
    let view = view_mat(camera.position, camera.rotation);
    let model = model_mat(PLANE_POSITION, vec3f(instance.size, 1), quat(vec3f(1, 0, 0), PI / 2));
    let clip_position = projection * view * model * vec4f(vertex.position, 1);
    return Fragment(clip_position, instance.color);
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4f {
    return fragment.color;
}
