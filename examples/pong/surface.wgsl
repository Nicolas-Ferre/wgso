#mod main

// TODO: rename and move function in std
// TODO: convert constants in vec2f + move in config.wgsl

const MIN_WIDTH = 1.9;
const MIN_HEIGHT = 1.2;

fn surface_ratio(surface_size: vec2u) -> vec2f {
    let ratio = f32(surface_size.x) / f32(surface_size.y);
    return select(
        vec2f(1, ratio) / (MIN_WIDTH / 2),
        vec2f(1 / ratio, 1) / (MIN_HEIGHT / 2),
        ratio > MIN_WIDTH / MIN_HEIGHT
    );
}
