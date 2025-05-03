#init init()
#run<42> test_compute(mode=mode0)
#run<-42> test_compute(mode=mode1)
#draw<-42> test_render<vertices>(instance=instance2)
#draw<42> test_render<vertices>(instance=instance1)
#init test_compute(mode=modes.inner.mode0)

var<storage, read_write> mode0: u32;
var<storage, read_write> mode1: u32;
var<storage, read_write> modes: ModeContainer;
var<storage, read_write> instance1: Instance;
var<storage, read_write> instance2: Instance;
var<storage, read_write> vertices: array<Vertex, 6>;

struct ModeContainer {
    alignment: array<u32, 64>,
    inner: Modes,
}

struct Modes {
    mode0: u32,
    mode1: u32,
}

struct Vertex {
    position: vec2f,
}

struct Instance {
    position: vec2f,
    color: vec4f,
}
