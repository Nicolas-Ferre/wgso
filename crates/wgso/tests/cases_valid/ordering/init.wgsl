#shader<compute> init

#import main

@compute
@workgroup_size(1, 1, 1)
fn main() {
    mode0 = 0;
    mode1 = 1;
}
