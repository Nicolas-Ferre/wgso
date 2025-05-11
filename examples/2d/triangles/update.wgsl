#shader<compute> update_triangles

#import ~.storage
#import _.std.storage

@compute
@workgroup_size(TRIANGLE_COUNT, 1, 1)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>,) {
    let delta = std_.time.frame_delta_secs;
    triangles.instances[local_id.x].brightness_param += TRIANGLE_BRIGHTNESS_INCREMENT * delta;
}
