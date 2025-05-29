#init ~.init.init_cubes()
#init ~.init_instances.init_cube_instances()
#run ~.update.update()
#draw ~.draw.cube<cubes.vertices, cubes.instances>(camera=camera, material=cubes.material, light=light)

#import _.std.vertex
#import _.std.quaternion

const CUBE_COUNT_X = 16;
const CUBE_COUNT_Y = 16;
const CUBE_COUNT = CUBE_COUNT_X * CUBE_COUNT_Y;
const CUBE_SIZE = vec3f(1, 1, 1) * 0.5;

struct Cubes {
    material: CubeMaterial,
    vertices: array<Vertex, 36>,
    instances: array<Cube, CUBE_COUNT>,
}

struct Cube {
    position: vec3f,
    rotation: vec4f,
    color: vec4f,
}

struct CubeMaterial {
    specular_power: f32,
}
