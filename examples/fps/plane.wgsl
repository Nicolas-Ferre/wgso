#mod main
#init ~.init()
#draw ~.render<plane.vertices, plane.instance>(camera=camera)
#import _.std.vertex.type

struct Plane {
    vertices: array<Vertex, 6>,
    instance: PlaneInstance,
}

struct PlaneInstance {
    size: vec2f,
    color: vec4f,
}

#mod storage
#import ~.main

var<storage, read_write> plane: Plane;

#shader<compute> init
#import ~.storage
#import _.std.vertex.model

@compute
@workgroup_size(1, 1, 1)
fn main() {
    plane.vertices = rectangle_vertices();
    plane.instance.size = vec2f(10, 10);
    plane.instance.color = vec4f(1, 1, 1, 1);
}

#shader<render, Vertex, PlaneInstance> render
#import ~.main
#import camera.main
#import _.std.math.constant
#import _.std.math.matrix
#import _.std.vertex.type

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
    let view = view_mat(camera.position, vec3f(1, 1, 1), camera.rotation);
    let model = model_mat(PLANE_POSITION, vec3f(instance.size, 1), quat(vec3f(1, 0, 0), PI / 2));
    let clip_position = projection * view * model * vec4f(vertex.position, 1);
    return Fragment(clip_position, instance.color);
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4f {
    return fragment.color;
}
