#mod main
#init ~.init()
#init ~.init_instances()
#run ~.update()
#draw ~.render<cubes.vertices, cubes.instances>(camera=camera, material=cubes.material, light=light)
#import _.std.vertex.type
#import _.std.math.quaternion

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

#mod storage
#import ~.main

var<storage, read_write> cubes: Cubes;

#shader<compute> init
#import ~.storage
#import _.std.math.quaternion
#import _.std.vertex.model

@compute
@workgroup_size(1, 1, 1)
fn main() {
    cubes.vertices = cube_vertices();
    cubes.material.specular_power = 16;
}

#shader<compute> init_instances
#import ~.storage
#import _.std.math.random
#import _.std.math.quaternion
#import _.std.state.storage

@compute
@workgroup_size(CUBE_COUNT_X, CUBE_COUNT_Y, 1)
fn main(
    @builtin(local_invocation_id) id: vec3u,
    @builtin(local_invocation_index) index: u32,
) {
    var seed = std_.time.start_secs * (1 + index);
    cubes.instances[index].position = vec3f(
        (f32(id.x) - CUBE_COUNT_X / 2) * CUBE_SIZE.x * 2,
        (f32(id.y) - CUBE_COUNT_Y / 2) * CUBE_SIZE.y * 2,
        0,
    );
    cubes.instances[index].rotation = DEFAULT_QUAT;
    cubes.instances[index].color = vec4f(
        normalize(vec3f(
            random_f32(&seed, 0, 1),
            random_f32(&seed, 0, 1),
            random_f32(&seed, 0, 1),
        )),
        1,
    );
}

#shader<compute> update
#import ~.storage
#import _.std.state.storage

@compute
@workgroup_size(CUBE_COUNT_X, CUBE_COUNT_Y, 1)
fn main(@builtin(local_invocation_index) index: u32) {
    let instance = &cubes.instances[index];
    let delta = std_.time.frame_delta_secs;
    instance.rotation = quat_mul(instance.rotation, quat(vec3f(1, 0, 0), 0.30 * delta));
    instance.rotation = quat_mul(instance.rotation, quat(vec3f(0, 1, 0), 0.5 * delta));
    instance.rotation = quat_mul(instance.rotation, quat(vec3f(0, 0, 1), 0.6 * delta));
}

#shader<render, Vertex, Cube> render
#import ~.main
#import camera.main
#import light.main
#import _.std.color.lighting
#import _.std.vertex.transform
#import _.std.vertex.type

var<uniform> camera: Camera;
var<uniform> material: CubeMaterial;
var<uniform> light: Light;

struct Fragment {
    @builtin(position)
    clip_position: vec4f,
    @location(0)
    world_position: vec3f,
    @location(1)
    world_normal: vec3f,
    @location(2)
    color: vec4f,
}

@vertex
fn vs_main(vertex: Vertex, instance: Cube) -> Fragment {
    let projection = proj_mat(camera.surface_ratio, camera.fov, camera.far, camera.near);
    let view = view_mat(camera.position, vec3f(1, 1, 1), camera.rotation);
    let model = model_mat(instance.position, CUBE_SIZE, instance.rotation);
    let clip_position = projection * view * model * vec4f(vertex.position, 1);
    let world_position = model * vec4f(vertex.position, 1);
    let world_normal = model * vec4f(vertex.normal, 0);
    return Fragment(clip_position, world_position.xyz, world_normal.xyz, instance.color);
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4f {
    let diffuse_strength = diffuse_strength(
        fragment.world_position,
        fragment.world_normal,
        light.point.position,
    );
    let specular_strength = specular_strength(
        fragment.world_position,
        fragment.world_normal,
        light.point.position,
        camera.position,
        material.specular_power,
    );
    let light_strength = light.ambient.strength + diffuse_strength + specular_strength;
    return vec4f(fragment.color.rgb * light.point.color * light_strength, fragment.color.a);
}
