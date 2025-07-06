#mod main

struct Field {
    color_angle: f32,
    z: f32,
}

#mod state
#import ~.main
#import _.std.state.storage

const _FIELD_BORDER_COLOR_SPEED = PI / 2;

fn init_field(z: f32) -> Field {
    return Field(0, z);
}

fn update_field(field: Field) -> Field {
    var updated = field;
    updated.color_angle += std_.time.frame_delta_secs * _FIELD_BORDER_COLOR_SPEED;
    if updated.color_angle > 2 * PI {
        updated.color_angle -= 2 * PI;
    }
    return updated;
}

#shader<render, Vertex, Field> render
#import ~.main
#import config.constant
#import _.std.math.distance
#import _.std.state.type
#import _.std.vertex.transform
#import _.std.vertex.type

const SHAPE_FACTOR = 10.;
const BORDER_GLOW_FACTOR = 0.002;
const BORDER_THICKNESS = 0.02;
const SEPARATOR_COLOR = vec3f(1, 1, 1);
const SEPARATOR_HEIGHT = 0.9;
const SEPARATOR_THICKNESS = 0.0025;
const SEPARATOR_GLOW_FACTOR = 0.001;

var<uniform> surface: Surface;

struct Fragment {
    @builtin(position)
    clip_position: vec4f,
    @location(0)
    world_position: vec2f,
    @location(1)
    color_angle: f32,
}

@vertex
fn vs_main(vertex: Vertex, instance: Field) -> Fragment {
    let scale_factor = scale_factor(surface.size, VISIBLE_AREA_MIN_SIZE);
    let position = vertex.position.xy * FIELD_SIZE * SHAPE_FACTOR;
    return Fragment(
        vec4f(position * scale_factor, instance.z, 1),
        position,
        instance.color_angle,
    );
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4f {
    return vec4f(border_color(fragment) + separator_color(fragment), 1.);
}

fn border_color(fragment: Fragment) -> vec3f {
    let angle = angle_vec2f(fragment.world_position, vec2f(1.));
    let rotated_angle = angle + fragment.color_angle;
    let dist = abs(rect_signed_dist(fragment.world_position, FIELD_SIZE + vec2f(BORDER_THICKNESS)));
    let brightness = brightness(dist, BORDER_THICKNESS, BORDER_GLOW_FACTOR);
    let color = color(rotated_angle);
    return brightness * color;
}

fn separator_color(fragment: Fragment) -> vec3f {
    const HALF_SIZE = FIELD_SIZE.y * SEPARATOR_HEIGHT / 2;
    let dist = segment_signed_dist(fragment.world_position, vec2f(0, -HALF_SIZE), vec2f(0, HALF_SIZE));
    return brightness(dist, SEPARATOR_THICKNESS, SEPARATOR_GLOW_FACTOR) * SEPARATOR_COLOR;
}

fn brightness(signed_dist: f32, thickness: f32, glow_factor: f32) -> f32 {
    let exterior_factor = clamp(glow_factor / (signed_dist - thickness / 2), 0, 1);
    let interior_factor = step(signed_dist, thickness / 2);
    return exterior_factor + interior_factor;
}

fn color(angle: f32) -> vec3f {
    const a = vec3f(0.50, 0.50, 0.50);
    const b = vec3f(0.50, 0.50, 0.50);
    const c = vec3f(1.00, 1.00, 1.00);
    const d = vec3f(0.00, 0.33, 0.67);
    let normalized_angle = angle / (2. * PI);
    return a + b * cos(6.283185 * (c * normalized_angle + d));
}
