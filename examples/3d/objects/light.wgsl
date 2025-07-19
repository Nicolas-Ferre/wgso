#mod main
#import _.std.color.constant

const LIGHT_COLOR = WHITE;

struct Light {
    position: vec3f,
    ambient_strength: f32,
}

#mod compute
#import ~.main

fn init_light(position: vec3f, ambient_strength: f32) -> Light {
    return Light(position, ambient_strength);
}
