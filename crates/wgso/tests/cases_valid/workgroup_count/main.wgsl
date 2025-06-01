#mod main

struct MaxInvocationIds {
    x: atomic<u32>,
    y: atomic<u32>,
    z: atomic<u32>,
}

#shader<compute, 10> x
#run ~.x()
#import ~.main

var<storage, read_write> max_invocation_ids_x: MaxInvocationIds;

@compute
@workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    atomicMax(&max_invocation_ids_x.x, global_id.x);
    atomicMax(&max_invocation_ids_x.y, global_id.y);
    atomicMax(&max_invocation_ids_x.z, global_id.z);
}

#shader<compute, 10, 15> xy
#run ~.xy()
#import ~.main

var<storage, read_write> max_invocation_ids_xy: MaxInvocationIds;

@compute
@workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    atomicMax(&max_invocation_ids_xy.x, global_id.x);
    atomicMax(&max_invocation_ids_xy.y, global_id.y);
    atomicMax(&max_invocation_ids_xy.z, global_id.z);
}

#shader<compute, 10, 15, 20> xyz
#run ~.xyz()
#import ~.main

var<storage, read_write> max_invocation_ids_xyz: MaxInvocationIds;

@compute
@workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    atomicMax(&max_invocation_ids_xyz.x, global_id.x);
    atomicMax(&max_invocation_ids_xyz.y, global_id.y);
    atomicMax(&max_invocation_ids_xyz.z, global_id.z);
}
