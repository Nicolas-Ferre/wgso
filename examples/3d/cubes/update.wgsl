#shader<compute> update

#import ~.storage

@compute
@workgroup_size(CUBE_COUNT_X, CUBE_COUNT_Y, 1)
fn main(@builtin(local_invocation_index) index: u32) {
    cubes.instances[index].rotation = quat_mul(cubes.instances[index].rotation, quat(vec3f(1, 0, 0), 0.005));
    cubes.instances[index].rotation = quat_mul(cubes.instances[index].rotation, quat(vec3f(0, 1, 0), 0.008));
    cubes.instances[index].rotation = quat_mul(cubes.instances[index].rotation, quat(vec3f(0, 0, 1), 0.010));
}
