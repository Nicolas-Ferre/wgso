#init init_triangles()
#run update_triangles()
#draw triangle<vertices.triangle, triangles.instances>()

struct Triangle {
    position: vec2f,
    brightness_param: f32,
}

fn triangle_brightness(brightness_param: f32) -> f32 {
    return (sin(brightness_param / 3.14 * 5) + 0.5) / 2 + 0.5;
}
