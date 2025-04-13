#shader<compute> orchestrator

var<storage, read_write> mode0: u32;
var<storage, read_write> mode1: u32;

@compute
@workgroup_size(1, 1, 1)
fn main() {
    mode0 = 0;
    mode1 = 1;
}

#run orchestrator()
#run test_compute(mode=mode0)
#run test_compute(mode=mode1)
#run test_compute(mode=mode0)
