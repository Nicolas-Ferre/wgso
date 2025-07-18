#mod main
#import objects.camera.compute
#import objects.plane.compute
#import _.std.vertex.model

#init ~.init()
#run ~.update()
#draw objects.plane.render<main.rectangle_vertices, main.plane>(camera=main.camera, surface=std_.surface)

struct Main {
    camera: Camera,
    plane: Plane,
    rectangle_vertices: array<Vertex, 6>,
}

var<storage, read_write> main: Main;

#shader<compute> init
#import ~.main

@compute
@workgroup_size(1, 1, 1)
fn main_() {
    main.camera = init_camera();
    main.plane = init_plane();
    main.rectangle_vertices = rectangle_vertices();
}

#shader<compute> update
#import ~.main

@compute
@workgroup_size(1, 1, 1)
fn main_() {
    main.camera = update_camera(main.camera);
}
