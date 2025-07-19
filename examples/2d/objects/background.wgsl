#mod main

struct Background {
    z: f32,
}

#mod compute
#import ~.main

fn init_background(z: f32) -> Background {
    return Background(z);
}

#shader<render, Vertex, Background> render
#import ~.main
#import _.std.color.constant
#import _.std.vertex.type

const VARIATION_SPEED = 1;
const MAX_BRIGHTNESS = 1. / 30;

var<uniform> time_secs: f32;

struct Fragment {
    @builtin(position)
    position: vec4f,
    @location(0)
    color: vec4f,
};

@vertex
fn vs_main(vertex: Vertex, instance: Background) -> Fragment {
    let brightness = (cos(time_secs * VARIATION_SPEED) + 1) / 2 * MAX_BRIGHTNESS;
    return Fragment(
        vec4f(vertex.position.xy * 2, instance.z, 1.),
        vec4f(WHITE.rgb * brightness, 1.),
    );
}

@fragment
fn fs_main(frag: Fragment) -> @location(0) vec4f {
    return frag.color;
}
