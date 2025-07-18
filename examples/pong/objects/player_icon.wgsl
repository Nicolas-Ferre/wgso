#mod main

struct PlayerIcon {
    position: vec3f,
    size: f32,
    button_state: u32,
}

#mod compute
#import ~.main
#import _.std.ui.main

fn init_player_icon(position: vec3f, size: f32) -> PlayerIcon {
    return PlayerIcon(position, size, BUTTON_STATE_NONE);
}

#shader<render, Vertex, PlayerIcon> render
#import ~.main
#import constant.main
#import _.std.color.constant
#import _.std.math.distance
#import _.std.io.main
#import _.std.ui.main
#import _.std.vertex.transform
#import _.std.vertex.type

const SIZE_RATIO = vec2f(1.0, 0.6);
const COLOR = CYAN;
const BORDER_COLOR = BLACK;
const BORDER_THICKNESS = 0.02;
const HEAD_POSITION = vec2f(0, 0.1);
const HEAD_RADIUS = 0.2;
const BODY_POSITION = vec2f(0, -0.49);
const BODY_RADIUS = 0.4;

var<uniform> surface: Surface;

struct Fragment {
    @builtin(position)
    clip_position: vec4f,
    @location(0)
    relative_position: vec2f,
    @location(1)
    brightness: f32,
}

@vertex
fn vs_main(vertex: Vertex, instance: PlayerIcon) -> Fragment {
    let scale_factor = scale_factor(surface.size, VISIBLE_AREA_MIN_SIZE);
    let size = instance.size * SIZE_RATIO;
    let position = vertex.position.xy * size + instance.position.xy;
    return Fragment(
        vec4f(position * scale_factor, instance.position.z, 1),
        vertex.position.xy,
        brightness(instance.button_state),
    );
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4f {
    if distance(fragment.relative_position * SIZE_RATIO, HEAD_POSITION) < HEAD_RADIUS {
        return vec4f(fragment.brightness, COLOR.gb, 1.);
    }
    if distance(fragment.relative_position * SIZE_RATIO, BODY_POSITION) < BODY_RADIUS {
        return vec4f(fragment.brightness, COLOR.gb, 1.);
    }
    if distance(fragment.relative_position * SIZE_RATIO, HEAD_POSITION) < HEAD_RADIUS + BORDER_THICKNESS {
        return BORDER_COLOR;
    }
    if distance(fragment.relative_position * SIZE_RATIO, BODY_POSITION) > BODY_RADIUS + BORDER_THICKNESS {
        return INVISIBLE;
    }
    return BORDER_COLOR;
}

fn brightness(button_state: u32) -> f32 {
    if (button_state == BUTTON_STATE_PRESSED) {
        return 0.3;
    } else if (button_state == BUTTON_STATE_HOVERED) {
        return 0.15;
    } else {
        return 0;
    }
}
