#shader<compute> update_field

#import ~.storage
#import _.std.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    field.instance.time += std_.time.frame_delta_secs;
}
