#mod<render, Vertex, Cube> cube

#import ~.main
#import camera.main
#import light.main
#import _.std.lighting
#import _.std.matrix
#import _.std.vertex

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
    let view = view_mat(camera.position, camera.rotation);
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
    let light_strength = light.ambiant.strength + diffuse_strength + specular_strength;
    return vec4f(fragment.color.rgb * light.point.color * light_strength, fragment.color.a);
}
