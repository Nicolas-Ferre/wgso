#shader<compute> test
#shader<compute> test_alias

const BUFFER_SIZE = 10;

@group(0)
@binding(0)
var<storage, read_write> buffer: array<u32, BUFFER_SIZE>;

@compute
@workgroup_size(BUFFER_SIZE, 1, 1)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>) {
    buffer[local_id.x] += local_id.x * 2;
}
