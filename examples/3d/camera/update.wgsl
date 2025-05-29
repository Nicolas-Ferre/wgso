#mod<compute> update_camera

#import ~.storage
#import _.std.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    camera.surface_ratio = f32(std_.surface.size.x) / f32(std_.surface.size.y);
}
