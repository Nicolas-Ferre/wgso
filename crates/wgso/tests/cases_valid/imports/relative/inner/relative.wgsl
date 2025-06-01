#shader<compute> use_relative_import
#run ~.use_relative_import()

#import ~.side
#import ~.~.~.~.root.main

var<storage, read_write> relative1: u32;
var<storage, read_write> relative2: u32;

@compute
@workgroup_size(1, 1, 1)
fn main() {
    relative1 = CONSTANT1;
    relative2 = CONSTANT2;
}

#mod side
const CONSTANT2 = 3;
