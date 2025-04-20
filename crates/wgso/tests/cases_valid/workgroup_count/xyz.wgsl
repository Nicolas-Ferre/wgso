#shader<compute, 10, 15, 20> xyz
#run xyz()

var<storage, read_write> max_invocation_ids_xyz: MaxInvocationIds;

struct MaxInvocationIds {
    x: atomic<u32>,
    y: atomic<u32>,
    z: atomic<u32>,
}

@compute
@workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    atomicMax(&max_invocation_ids_xyz.x, global_id.x);
    atomicMax(&max_invocation_ids_xyz.y, global_id.y);
    atomicMax(&max_invocation_ids_xyz.z, global_id.z);
}
