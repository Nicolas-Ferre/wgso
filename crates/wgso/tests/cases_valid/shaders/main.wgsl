#mod main
#init ~.init()
#run<-42> ~.compute(mode=mode0)
#run<42> main.compute(mode=mode1)
#draw<-42> main.render<vertices, instance1>()
#draw<42> ~.render<vertices, instance2>()
#init ~.~.main.compute(mode=modes.inner.mode0)

var<storage, read_write> mode0: u32;
var<storage, read_write> mode1: u32;
var<storage, read_write> modes: ModeContainer;
var<storage, read_write> instance1: Instance;
var<storage, read_write> instance2: array<Instance, 1>;
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

#shader<compute> init
#import ~.main

@compute
@workgroup_size(1, 1, 1)
fn main() {
    mode0 = 0;
    mode1 = 1;
    modes = ModeContainer(array<u32, 64>(), Modes(0, 1));
    for (var i = 0; i < 64; i++) {
        modes.alignment[i] = 1;
    }
    instance1 = Instance(vec2f(-0.25, -0.25), vec4f(1., 1., 1., 1.));
    instance2 = array(Instance(vec2f(0.25, 0.25), vec4f(1., 0., 1., 1.)));
    vertices = array(
        Vertex(vec2f(-1, -1)),
        Vertex(vec2f(-1, 1)),
        Vertex(vec2f(1, -1)),
        Vertex(vec2f(1, 1)),
        Vertex(vec2f(1, -1)),
        Vertex(vec2f(-1, 1)),
    );
}

#shader<compute> compute

var<storage, read_write> buffer: i32;

var<uniform> mode: u32;

@compute
@workgroup_size(1, 1, 1)
fn main() {
    if mode == 0 {
        buffer += 5;
    } else {
        buffer *= 3;
    }
}

#shader<render, Vertex, Instance> render

struct Vertex {
    position: vec2f,
}

struct Instance {
    position: vec2f,
    color: vec4f,
}

struct Fragment {
    @builtin(position)
    position: vec4f,
    @location(0)
    color: vec4f,
};

@vertex
fn vs_main(vertex: Vertex, instance: Instance) -> Fragment {
    return Fragment(vec4f(vertex.position / 2. + instance.position, 0., 1.), instance.color);
}

@fragment
fn fs_main(frag: Fragment) -> @location(0) vec4f {
    return frag.color;
}
