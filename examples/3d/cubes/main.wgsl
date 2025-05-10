#init init_cubes()
#init init_cube_instances()
#run update()
#draw cube<cubes.vertices, cubes.instances>(camera=camera, light=light)

#import _.std.vertex
#import _.std.quaternion

const CUBE_COUNT_X = 16;
const CUBE_COUNT_Y = 16;
const CUBE_COUNT = CUBE_COUNT_X * CUBE_COUNT_Y;
const CUBE_SIZE = vec3f(0.5, 0.5, 0.5);

struct Cubes {
    vertices: array<Vertex, 36>,
    instances: array<Cube, CUBE_COUNT>,
}

struct Cube {
    position: vec3f,
    rotation: vec4f,
    color: vec4f,
    specular_power: f32,
}
