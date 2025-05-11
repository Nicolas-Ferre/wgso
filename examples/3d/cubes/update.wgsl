#shader<compute> update

#import ~.storage
#import _.std.storage

@compute
@workgroup_size(CUBE_COUNT_X, CUBE_COUNT_Y, 1)
fn main(@builtin(local_invocation_index) index: u32) {
    let instance = &cubes.instances[index];
    let delta = std_.time.frame_delta_secs;
    instance.rotation = quat_mul(instance.rotation, quat(vec3f(1, 0, 0), 0.30 * delta));
    instance.rotation = quat_mul(instance.rotation, quat(vec3f(0, 1, 0), 0.5 * delta));
    instance.rotation = quat_mul(instance.rotation, quat(vec3f(0, 0, 1), 0.6 * delta));
}
