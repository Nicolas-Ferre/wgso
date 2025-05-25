#shader<compute> update_surface

#import ~.storage
#import _.std.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    surface.size = std_.surface.size;
}
