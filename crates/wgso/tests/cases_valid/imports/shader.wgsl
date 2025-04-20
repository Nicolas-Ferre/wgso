#shader<compute> test
#run test()

#import inner.function

@compute
@workgroup_size(1, 1, 1)
fn main() {
    increment();
}
