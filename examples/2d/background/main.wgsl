#init ~.init.init_background()
#run ~.update.update_background()
#draw ~.draw.background<background.vertices, background.instance>()

#import _.std.vertex

const BACKGROUND_SPEED = 1;
const BACKGROUND_MAX_BRIGHTNESS = 1. / 30;

struct Background {
    vertices: array<Vertex, 6>,
    instance: BackgroundInstance,
    brightness_param: f32,
}

struct BackgroundInstance {
    color: vec4f,
}
