#shader<render, Vertex, UiButton> render
#import config.constant
#import _.std.color.constant
#import _.std.math.distance
#import _.std.state.type
#import _.std.ui.type
#import _.std.vertex.transform
#import _.std.vertex.type

const COLOR = CYAN;
const CORNER_RADIUS = 0.1;
const THICKNESS = 0.02;
const DISABLED_GLOW_FACTOR = 0.003;
const HOVERED_GLOW_FACTOR = DISABLED_GLOW_FACTOR * 2;
const PRESSED_GLOW_FACTOR = DISABLED_GLOW_FACTOR * 3;

var<uniform> surface: Surface;

struct Fragment {
    @builtin(position)
    clip_position: vec4f,
    @location(0)
    relative_position: vec2f,
    @location(1)
    glow_factor: f32,
}

@vertex
fn vs_main(vertex: Vertex, instance: UiButton) -> Fragment {
    let scale_factor = scale_factor(surface.size, VISIBLE_AREA_MIN_SIZE);
    let position = vertex.position.xy * instance.size + instance.position.xy;
    return Fragment(
        vec4f(position * scale_factor, instance.position.z, 1),
        vertex.position.xy,
        glow_factor(instance.state),
    );
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4f {
    let dist = rect_signed_dist(fragment.relative_position, vec2f(0.9, 0.9) - CORNER_RADIUS * 2) - CORNER_RADIUS;
    let exterior_brighness = step(0, dist) * exp(-dist / fragment.glow_factor);
    let interior_brighness = step(THICKNESS, -dist) * exp((dist + THICKNESS) / fragment.glow_factor);
    let middle_brightness = step(-dist, THICKNESS) * step(dist, 0);
    return COLOR * max(exterior_brighness, max(interior_brighness, middle_brightness));
}

fn glow_factor(button_state: u32) -> f32 {
    if (button_state == BUTTON_STATE_PRESSED) {
        return PRESSED_GLOW_FACTOR;
    } else if (button_state == BUTTON_STATE_HOVERED) {
        return HOVERED_GLOW_FACTOR;
    } else {
        return DISABLED_GLOW_FACTOR;
    }
}
