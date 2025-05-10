#shader<compute> init_cube_instances

#import ~.storage
#import _.std.quaternion

@compute
@workgroup_size(CUBE_COUNT_X, CUBE_COUNT_Y, 1)
fn main(
    @builtin(local_invocation_id) id: vec3u,
    @builtin(local_invocation_index) index: u32,
) {
    cubes.instances[index].position = vec3f(
        (f32(id.x) - CUBE_COUNT_X / 2) * CUBE_SIZE.x * 2,
        (f32(id.y) - CUBE_COUNT_Y / 2) * CUBE_SIZE.y * 2,
        0,
    );
    cubes.instances[index].rotation = DEFAULT_QUAT;
    cubes.instances[index].color = vec4f(0.2, 0.2, 1, 1);
    cubes.instances[index].specular_power = 16;
}
