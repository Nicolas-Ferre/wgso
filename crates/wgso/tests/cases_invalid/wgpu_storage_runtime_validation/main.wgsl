#mod<compute> test
#run test(param=buffer)

var<storage, read_write> buffer: u32;

var<uniform> param: u32;

@compute
@workgroup_size(1, 1, 1)
fn main() {
    buffer = param;
}
