#shader<compute> update_triangles

#import constants
#import triangles.storages

@compute
@workgroup_size(1, 1, 1)
fn main() {
    triangles.instance1.brightness_param += TRIANGLE_BRIGHTNESS_INCREMENT;
    triangles.instance2.brightness_param += TRIANGLE_BRIGHTNESS_INCREMENT;
    triangles.instance3.brightness_param += TRIANGLE_BRIGHTNESS_INCREMENT;
}
