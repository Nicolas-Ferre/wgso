#init init()
#run test_compute(mode=mode0)
#run test_compute(mode=mode1)
#run test_compute(mode=modes.inner.mode0)

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
