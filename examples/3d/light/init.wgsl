#shader<compute> init_light

#import ~.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    light.ambiant.strength = 0.05;
    light.point.position = vec3f(0.6, 0.6, -0.5);
    light.point.color = vec3f(1, 1, 1);
}
