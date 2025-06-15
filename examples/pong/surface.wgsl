#mod main
#run ~.update()

const RATIO_HORIZONTAL_FACTOR = 1.3;

struct SurfaceData {
    size: vec2u,
}

fn surface_ratio(surface_size: vec2u) -> vec2f {
    let ratio = f32(surface_size.x) / f32(surface_size.y);
    return select(
        vec2f(1, ratio) / 1,
        vec2f(1 / ratio, 1) * RATIO_HORIZONTAL_FACTOR,
        ratio > RATIO_HORIZONTAL_FACTOR
    );
}

#mod storage
#import ~.main

var<storage, read_write> surface: SurfaceData;

#shader<compute> update
#import ~.storage
#import _.std.state.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    surface.size = std_.surface.size;
}
