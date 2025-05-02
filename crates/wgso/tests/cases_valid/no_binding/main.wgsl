#shader<compute> test
#run test()

@compute
@workgroup_size(1, 1, 1)
fn main() {}
