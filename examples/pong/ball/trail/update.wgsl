#shader<compute, 10> update_ball_trail

#import ~.storage
#import ~.~.storage
#import _.std.storage

const THREAD_COUNT = MAX_BALL_TRAIL_PARTICLES / 10;
const SIZE_DECREASE_SPEED = 0.03;

@compute
@workgroup_size(THREAD_COUNT, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    ball_trail.particles[index].radius -= length(ball.instance.velocity) * SIZE_DECREASE_SPEED * std_.time.frame_delta_secs;
}
