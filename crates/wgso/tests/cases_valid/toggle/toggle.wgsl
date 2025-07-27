#mod main

#init ~.init()
#run ~.update()

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
