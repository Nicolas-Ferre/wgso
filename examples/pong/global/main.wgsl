#run update_surface()

struct Global {
    surface_size: vec2u,
    elapsed_secs: f32,
}

fn surface_ratio(surface_size: vec2u) -> vec2f {
    let ratio = f32(surface_size.x) / f32(surface_size.y);
    return select(vec2f(1, ratio), vec2f(1 / ratio, 1), ratio > 1);
}
