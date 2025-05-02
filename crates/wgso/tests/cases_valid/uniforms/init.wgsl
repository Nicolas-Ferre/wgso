#shader<compute> init

#import main

@compute
@workgroup_size(1, 1, 1)
fn main() {
    mode0 = 0;
    mode1 = 1;
    modes = ModeContainer(array<u32, 64>(), Modes(0, 1));
    for (var i = 0; i < 64; i++) {
        modes.alignment[i] = 1;
    }
}
