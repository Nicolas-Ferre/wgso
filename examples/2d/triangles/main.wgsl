#init init_triangles()
#run update_triangles()
#draw triangle<triangles.vertices, triangles.instances>()

#import _.std.vertex

const TRIANGLE_COUNT = 3;
const TRIANGLE_BRIGHTNESS_INCREMENT = 0.05;

struct Triangles {
    vertices: array<Vertex, 3>,
    instances: array<Triangle, TRIANGLE_COUNT>,
}

struct Triangle {
    position: vec2f,
    brightness_param: f32,
}

fn triangle_brightness(brightness_param: f32) -> f32 {
    return (sin(brightness_param / 3.14 * 5) + 0.5) / 2 + 0.5;
}
