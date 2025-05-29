#mod<compute> test_compute
#run test_compute()

@compute
@workgroup_size(1, 1, 1)
fn main() {}
