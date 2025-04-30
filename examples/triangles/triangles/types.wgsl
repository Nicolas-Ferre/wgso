struct TriangleState {
     instance1: Triangle,
     _padding1: array<u32, 60>,
     instance2: Triangle,
     _padding2: array<u32, 60>,
     instance3: Triangle,
}

struct Triangle {
    position: vec2f,
    brightness_param: f32,
}

fn triangle_brightness(brightness_param: f32) -> f32 {
    return (sin(brightness_param / 3.14 * 5) + 0.5) / 2 + 0.5;
}
