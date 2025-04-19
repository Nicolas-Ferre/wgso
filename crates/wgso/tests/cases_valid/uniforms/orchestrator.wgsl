#shader<compute> orchestrator

var<storage, read_write> mode0: u32;
var<storage, read_write> mode1: u32;
var<storage, read_write> modes: ModeContainer;

struct ModeContainer {
    alignment: array<u32, 64>,
    inner: Modes,
}

struct Modes {
    mode0: u32,
    mode1: u32,
}

@compute
@workgroup_size(1, 1, 1)
fn main() {
    mode0 = 0;
    mode1 = 1;
    modes = ModeContainer(array<u32, 64>(), Modes(0, 1));
    for (var i = 0; i < 64; i++) {
        modes.alignment[i] = 1;
    }
}

#run orchestrator()
#run test_compute(mode=mode0)
#run test_compute(mode=mode1)
#run test_compute(mode=modes.inner.mode0)
