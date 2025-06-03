#mod main
#run ~.update()

// TODO: specialize each elasped time to avoid precision drop
struct Global {
    surface_size: vec2u,
    elapsed_secs: f32,
}

fn surface_ratio(surface_size: vec2u) -> vec2f {
    let ratio = f32(surface_size.x) / f32(surface_size.y);
    return select(vec2f(1, ratio), vec2f(1 / ratio, 1), ratio > 1);
}

#mod storage
#import ~.main

var<storage, read_write> global: Global;

#shader<compute> update
#import ~.storage
#import _.std.math.constant
#import _.std.state.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    global.surface_size = std_.surface.size;
    global.elapsed_secs += std_.time.frame_delta_secs;
}
