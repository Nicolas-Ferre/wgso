#shader<compute> test
#run test(value=buffer)

var<storage, read_write> buffer: i32;

var<uniform> value: u32;

@compute
@workgroup_size(1, 1, 1)
fn main() {
    buffer = i32(value);
}
