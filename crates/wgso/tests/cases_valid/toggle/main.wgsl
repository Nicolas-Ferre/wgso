#mod state

#toggle<is_toggle_enabled> toggle
#run ~.update()

var<storage, read_write> state: u32;
var<storage, read_write> is_toggle_enabled: u32;

#shader<compute> update
#import ~.state

@compute
@workgroup_size(1, 1, 1)
fn main() {
    is_toggle_enabled = u32(!bool(is_toggle_enabled));
}
