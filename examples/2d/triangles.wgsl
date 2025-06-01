#mod main
#init ~.init()
#run ~.update()
#draw ~.render<triangles.vertices, triangles.instances>()
#import _.std.vertex.type

const TRIANGLE_COUNT = 3;
const TRIANGLE_BRIGHTNESS_INCREMENT = 3;

struct Triangles {
    vertices: array<Vertex, 3>,
    instances: array<Triangle, TRIANGLE_COUNT>,
}

struct Triangle {
    position: vec2f,
    brightness_param: f32,
}

fn triangle_brightness(brightness_param: f32) -> f32 {
    return (sin(brightness_param / 3.14 * 5) + 0.5) / 2 + 0.5;
}

#mod storage
#import ~.main

var<storage, read_write> triangles: Triangles;

#shader<compute> init
#import ~.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    triangles.vertices = array(
        Vertex(vec3f(0., 0.5, 0), vec3f(0, 0, 1)),
        Vertex(vec3f(-0.5, -0.5, 0), vec3f(0, 0, 1)),
        Vertex(vec3f(0.5, -0.5, 0), vec3f(0, 0, 1)),
    );
    triangles.instances = array(
        Triangle(vec2f(0.25, -0.25), 3.14 / 4),
        Triangle(vec2f(0., 0.), 3.14 / 8),
        Triangle(vec2f(-0.25, 0.25), 0.),
    );
}

#shader<compute> update
#import ~.storage
#import _.std.state.storage

@compute
@workgroup_size(TRIANGLE_COUNT, 1, 1)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>,) {
    let delta = std_.time.frame_delta_secs;
    triangles.instances[local_id.x].brightness_param += TRIANGLE_BRIGHTNESS_INCREMENT * delta;
}

#shader<render, Vertex, Triangle> render
#import ~.main
#import _.std.vertex.type

struct Fragment {
    @builtin(position)
    position: vec4f,
    @location(0)
    relative_position: vec4f,
    @location(1)
    brightness: f32,
};

@vertex
fn vs_main(vertex: Vertex, instance: Triangle) -> Fragment {
    let position = vec4f(vertex.position.xy + instance.position, 0., 1.);
    return Fragment(position, position, triangle_brightness(instance.brightness_param));
}

@fragment
fn fs_main(frag: Fragment) -> @location(0) vec4f {
    let red = (frag.relative_position.x + 1) / 2;
    let green = (frag.relative_position.y + 1) / 2;
    let blue = red * green;
    return vec4f(red, green, blue, 1) * frag.brightness;
}
