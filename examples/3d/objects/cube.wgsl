#mod main

struct Cube {
    position: vec3f,
    _phantom: vec2f, // workaround for an alignment bug on Android
    size: f32,
    rotation: vec4f,
    color: vec4f,
}

struct CubeMaterial {
    specular_power: f32,
}

#mod compute
#import ~.main
#import _.std.math.quaternion
#import _.std.math.random
#import _.std.io.compute

fn init_cube(position: vec3f, size: f32, color: vec4f) -> Cube {
    return Cube(position, vec2f(), size, DEFAULT_QUAT, color);
}

fn init_cube_on_grid(size: f32, grid_pos: vec2u, grid_size: vec2u) -> Cube {
    let index = grid_pos.y * grid_size.x + grid_pos.x;
    var seed = std_.time.start_secs * (1 + index);
    let position = vec3f((vec2f(grid_pos) - vec2f(grid_size) / 2) * size * 2, 0);
    let color = normalize(vec3f(
        random_f32(&seed, 0, 1),
        random_f32(&seed, 0, 1),
        random_f32(&seed, 0, 1),
    ));
    return Cube(position, vec2f(), size, DEFAULT_QUAT, vec4f(color, 1));
}

fn update_cube(cube: Cube) -> Cube {
    var updated = cube;
    let delta = std_.time.frame_delta_secs;
    updated.rotation = quat_mul(updated.rotation, quat(vec3f(1, 0, 0), 0.30 * delta));
    updated.rotation = quat_mul(updated.rotation, quat(vec3f(0, 1, 0), 0.5 * delta));
    updated.rotation = quat_mul(updated.rotation, quat(vec3f(0, 0, 1), 0.6 * delta));
    return updated;
}

fn init_cube_material(specular_power: f32) -> CubeMaterial {
    return CubeMaterial(specular_power);
}

#shader<render, Vertex, Cube> render
#import ~.main
#import objects.camera.main
#import objects.light.main
#import _.std.color.lighting
#import _.std.io.main
#import _.std.vertex.transform
#import _.std.vertex.type

var<uniform> camera: Camera;
var<uniform> material: CubeMaterial;
var<uniform> light: Light;
var<uniform> surface: Surface;

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
    let projection = proj_mat(surface.size, camera.fov, camera.far, camera.near);
    let view = view_mat(camera.position, vec3f(1, 1, 1), camera.rotation);
    let model = model_mat(instance.position, vec3f(instance.size), instance.rotation);
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
        light.position,
    );
    let specular_strength = specular_strength(
        fragment.world_position,
        fragment.world_normal,
        light.position,
        camera.position,
        material.specular_power,
    );
    let light_strength = light.ambient_strength + diffuse_strength + specular_strength;
    return vec4f(fragment.color.rgb * LIGHT_COLOR.rgb * light_strength, fragment.color.a);
}
