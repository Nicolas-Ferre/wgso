#shader<compute> test_compute

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
