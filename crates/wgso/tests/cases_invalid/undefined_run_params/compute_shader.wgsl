#shader<compute> test
#run test(value=buffer, undefined=buffer)

var<storage, read_write> buffer: u32;

var<uniform> value: u32;

@compute
@workgroup_size(1, 1, 1)
fn main() {
    buffer = value;
}
