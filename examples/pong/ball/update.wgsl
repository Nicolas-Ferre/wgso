#shader<compute> update_ball

#import ~.storage
#import field.main
#import _.std.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    const HALF_LIMIT = FIELD_SIZE / 2 - BALL_RADIUS;
    let instance = &ball.instance;
    instance.position += instance.velocity * std_.time.frame_delta_secs;
    instance.velocity = select(instance.velocity, -instance.velocity, abs(instance.position) > HALF_LIMIT);
    instance.position = clamp(instance.position, -HALF_LIMIT, HALF_LIMIT);
}
