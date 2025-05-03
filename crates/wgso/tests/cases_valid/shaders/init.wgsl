#shader<compute> init

#import main

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
