#mod main
#init ~.init()
#run ~.update()
#draw ~.render<background.vertices, background.instance>()
#import _.std.vertex.type

const BACKGROUND_SPEED = 1;
const BACKGROUND_MAX_BRIGHTNESS = 1. / 30;

struct Background {
    vertices: array<Vertex, 6>,
    instance: BackgroundInstance,
    brightness_param: f32,
}

struct BackgroundInstance {
    color: vec4f,
}

#mod storage
#import ~.main

var<storage, read_write> background: Background;

#shader<compute> init
#import ~.storage
#import _.std.vertex.model

@compute
@workgroup_size(1, 1, 1)
fn main() {
    background.vertices = rectangle_vertices();
}

#shader<compute> update
#import ~.storage
#import _.std.color.constant
#import _.std.state.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    let delta = std_.time.frame_delta_secs;
    background.brightness_param += delta * BACKGROUND_SPEED;
    let brightness = (cos(background.brightness_param) + 1) / 2 * BACKGROUND_MAX_BRIGHTNESS;
    background.instance.color = vec4f(WHITE.rgb * brightness, 1.);
}

#shader<render, Vertex, BackgroundInstance> render
#import ~.main
#import _.std.vertex.type

struct Fragment {
    @builtin(position)
    position: vec4f,
    @location(0)
    color: vec4f,
};

@vertex
fn vs_main(vertex: Vertex, instance: BackgroundInstance) -> Fragment {
    return Fragment(vec4f(vertex.position.xy * 2, 0.5, 1.), instance.color);
}

@fragment
fn fs_main(frag: Fragment) -> @location(0) vec4f {
    return frag.color;
}
