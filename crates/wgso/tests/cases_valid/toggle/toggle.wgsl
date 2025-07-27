#mod main

#init ~.init()
#run ~.update()
#draw toggle.render<vertices, instance>()

var<storage, read_write> toggle_state: u32;

#shader<compute> init
#import ~.main

@compute
@workgroup_size(1, 1, 1)
fn main() {
    toggle_state += 1;
}

#shader<compute> update
#import ~.main
#import main.state

@compute
@workgroup_size(1, 1, 1)
fn main() {
    state = toggle_state;
}

#shader<render, Vertex, Instance> render
#import main.instance
#import _.std.vertex.type

var<storage, read> toggle_state: u32;

struct Fragment {
    @builtin(position)
    position: vec4f,
};

@vertex
fn vs_main(vertex: Vertex, instance: Instance) -> Fragment {
    return Fragment(vec4f(vertex.position * 2, 1.));
}

@fragment
fn fs_main(frag: Fragment) -> @location(0) vec4f {
    return vec4f(f32(toggle_state), f32(toggle_state), f32(toggle_state), 1);
}
