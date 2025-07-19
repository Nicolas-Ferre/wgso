#mod main
#import _.std.vertex.type

struct Triangle {
    position: vec3f,
    time_offset_secs: f32,
}

fn triangle_vertices() -> array<Vertex, 3> {
    const normal = vec3f(0, 0, 1);
    const position1 = vec3f(0., 0.5, 0);
    const position2 = vec3f(-0.5, -0.5, 0);
    const position3 = vec3f(0.5, -0.5, 0);
    return array(
        Vertex(position1, normal),
        Vertex(position2, normal),
        Vertex(position3, normal),
    );
}

#mod compute
#import ~.main

fn init_triangle(position: vec3f, time_offset_secs: f32) -> Triangle {
    return Triangle(position, time_offset_secs);
}

#shader<render, Vertex, Triangle> render
#import ~.main
#import _.std.color.constant
#import _.std.math.constant
#import _.std.vertex.type

const VARIATION_SPEED = 3;

var<uniform> time_secs: f32;

struct Fragment {
    @builtin(position)
    position: vec4f,
    @location(0)
    world_position: vec4f,
    @location(1)
    brightness: f32,
};

@vertex
fn vs_main(vertex: Vertex, instance: Triangle) -> Fragment {
    let position = vec4f(vertex.position.xy + instance.position.xy, instance.position.z, 1.);
    let brightness = (sin((time_secs + instance.time_offset_secs) * VARIATION_SPEED / PI * 5) + 0.5) / 2 + 0.5;
    return Fragment(
        position,
        position,
        brightness,
    );
}

@fragment
fn fs_main(frag: Fragment) -> @location(0) vec4f {
    let red = (frag.world_position.x + 1) / 2;
    let green = (frag.world_position.y + 1) / 2;
    let blue = red * green;
    return vec4f(red, green, blue, 1) * frag.brightness;
}
