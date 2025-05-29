#mod<compute> update_background

#import ~.storage
#import _.std.color
#import _.std.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    let delta = std_.time.frame_delta_secs;
    background.brightness_param += delta * BACKGROUND_SPEED;
    let brightness = (cos(background.brightness_param) + 1) / 2 * BACKGROUND_MAX_BRIGHTNESS;
    background.instance.color = vec4f(WHITE.rgb * brightness, 1.);
}
