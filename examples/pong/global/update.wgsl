#shader<compute> update_surface

#import ~.storage
#import _.std.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    global.surface_size = std_.surface.size;
    global.elapsed_secs += std_.time.frame_delta_secs;
}
