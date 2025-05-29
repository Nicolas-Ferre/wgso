#shader<compute> update_ball

#import ~.storage
#import ~.trail.storage
#import field.main
#import global.storage
#import _.std.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    bounce_on_walls();
    generate_trail();
}

fn bounce_on_walls() {
    const HALF_LIMIT = FIELD_SIZE / 2 - BALL_RADIUS;
    let instance = &ball.instance;
    instance.position += instance.velocity * std_.time.frame_delta_secs;
    instance.velocity = select(instance.velocity, -instance.velocity, abs(instance.position) > HALF_LIMIT);
    instance.position = clamp(instance.position, -HALF_LIMIT, HALF_LIMIT);
}

fn generate_trail() {
    let millis = u32(global.elapsed_secs * length(ball.instance.velocity) * 60 / 1000);
    if millis % 2 == 0 {
        create_ball_trail_particle(ball.instance.position);
    }
}
