#mod main
#import objects.camera.compute
#import objects.cube.compute
#import objects.light.compute
#import _.std.vertex.type

#init ~.init()
#init ~.init_cubes()
#run ~.update_cubes()
#draw objects.cube.render<main.cube_vertices, main.cubes>(camera=main.camera, material=main.cube_material, light=main.light, surface=std_.surface)
#draw objects.cube.render<main.cube_vertices, main.light_cube>(camera=main.camera, material=main.cube_material, light=main.full_light, surface=std_.surface)

const GRID_SIZE = vec2u(16, 16);
const CUBE_COUNT = GRID_SIZE.x * GRID_SIZE.y;

struct Main {
    light: Light,
    _phantom1: array<f32, 60>,
    full_light: Light,
    _phantom2: array<f32, 60>,
    camera: Camera,
    _phantom3: array<f32, 52>,
    cube_material: CubeMaterial,
    cubes: array<Cube, CUBE_COUNT>,
    light_cube: Cube,
    cube_vertices: array<Vertex, 36>,
}

var<storage, read_write> main: Main;

#shader<compute> init
#import ~.main
#import _.std.color.constant
#import _.std.math.constant
#import _.std.math.quaternion
#import _.std.vertex.model

const LIGHT_AMBIENT_STRENGTH = 0.05;
const LIGHT_POSITION = vec3f(0.6, 0.6, -0.5);
const LIGHT_CUBE_SIZE = 0.05;
const CUBE_SPECULAR_POWER = 16;

@compute
@workgroup_size(1, 1, 1)
fn main_() {
    main.light = init_light(LIGHT_POSITION, LIGHT_AMBIENT_STRENGTH);
    main.full_light = init_light(vec3f(), 1);
    main.camera = init_camera();
    main.cube_material = init_cube_material(CUBE_SPECULAR_POWER);
    main.cube_vertices = cube_vertices();
    main.light_cube = init_cube(LIGHT_POSITION, LIGHT_CUBE_SIZE, WHITE);
}

#shader<compute> init_cubes
#import ~.main

const CUBE_SIZE = 0.5;

@compute
@workgroup_size(GRID_SIZE.x, GRID_SIZE.y, 1)
fn main_(
    @builtin(local_invocation_id) id: vec3u,
    @builtin(local_invocation_index) index: u32,
) {
    main.cubes[index] = init_cube_on_grid(CUBE_SIZE, id.xy, GRID_SIZE);
}

#shader<compute> update_cubes
#import ~.main

@compute
@workgroup_size(GRID_SIZE.x, GRID_SIZE.y, 1)
fn main_(@builtin(local_invocation_index) index: u32) {
    main.cubes[index] = update_cube(main.cubes[index]);
}
