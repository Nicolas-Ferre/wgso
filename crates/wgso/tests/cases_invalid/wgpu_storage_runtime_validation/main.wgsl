#shader<compute> test
#run ~.test()

var<storage, read_write> buffer: array<u32, 999999999>;

@compute
@workgroup_size(1, 1, 1)
fn main() {
    buffer[0] = buffer[1];
}
