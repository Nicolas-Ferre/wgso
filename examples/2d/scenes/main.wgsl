#mod main
#import objects.background.compute
#import objects.triangle.compute
#import _.std.vertex.model

#init ~.init()
#run ~.update()
#draw objects.background.render<main.rectangle_vertices, main.background>(time_secs=main.time_secs)
#draw objects.triangle.render<main.triangle_vertices, main.triangles>(time_secs=main.time_secs)

const BACKGROUND_Z = 0.9;
const TRIANGLE_Z = 0.8;

const TRIANGLE_COUNT = 3;

struct Main {
    time_secs: f32,
    rectangle_vertices: array<Vertex, 6>,
    triangle_vertices: array<Vertex, 3>,
    background: Background,
    triangles: array<Triangle, TRIANGLE_COUNT>
}

var<storage, read_write> main: Main;

#shader<compute> init
#import ~.main

const TRIANGLE_TIME_OFFSET_SECS_DIFF = -0.1;
const TRIANGLE_POSITIONS = array(vec2f(0.25, -0.25), vec2f(0., 0.), vec2f(-0.25, 0.25));

@compute
@workgroup_size(1, 1, 1)
fn main_() {
    main.time_secs = 0;
    main.rectangle_vertices = rectangle_vertices();
    main.triangle_vertices = triangle_vertices();
    main.background = init_background(BACKGROUND_Z);
    init_triangles();
}

fn init_triangles() {
    for (var i = 0u; i < TRIANGLE_COUNT; i++) {
        let position = vec3f(TRIANGLE_POSITIONS[i], TRIANGLE_Z);
        let time_offset_secs = f32(i) * TRIANGLE_TIME_OFFSET_SECS_DIFF;
        main.triangles[i] = init_triangle(position, time_offset_secs);
    }
}

#shader<compute> update
#import ~.main
#import _.std.io.compute

@compute
@workgroup_size(1, 1, 1)
fn main_() {
    main.time_secs += std_.time.frame_delta_secs;
}
