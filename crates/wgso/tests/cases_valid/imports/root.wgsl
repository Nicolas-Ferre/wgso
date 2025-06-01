#mod main
#run ~.test()

const CONSTANT1 = 2;

#shader<compute> test
#import inner.inner.function

@compute
@workgroup_size(1, 1, 1)
fn main() {
    increment();
}
