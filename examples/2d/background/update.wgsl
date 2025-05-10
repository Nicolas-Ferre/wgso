#shader<compute> update_background

#import ~.storage
#import triangles.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    let brightness_param = triangles.instances[1].brightness_param;
    let brightness = triangle_brightness(brightness_param * BACKGROUND_SPEED) * BACKGROUND_MAX_BRIGHTNESS;
    background.instance.color = vec4f(vec3f(1., 1., 1.) * brightness, 1.);
}
