#init init_background()
#run update_background()
#draw background<background.vertices, background.instance>()

#import _.std.vertex

const BACKGROUND_SPEED = 0.2;
const BACKGROUND_MAX_BRIGHTNESS = 1. / 50;

struct Background {
    vertices: array<Vertex, 6>,
    instance: BackgroundInstance,
}

struct BackgroundInstance {
    color: vec4f,
}
