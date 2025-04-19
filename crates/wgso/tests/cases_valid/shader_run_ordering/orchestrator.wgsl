#shader<compute> orchestrator

var<storage, read_write> mode0: u32;
var<storage, read_write> mode1: u32;

@compute
@workgroup_size(1, 1, 1)
fn main() {
    mode0 = 0;
    mode1 = 1;
}

#init orchestrator()
#run<42> test_compute(mode=mode0)
#run<-42> test_compute(mode=mode1)
#init test_compute(mode=mode0)
