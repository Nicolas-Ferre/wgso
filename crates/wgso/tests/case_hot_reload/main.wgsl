#init ~.init.init()
#draw ~.draw.rectangle<state.vertices, state.instance>()

struct Vertex {
    position: vec2f,
}

struct Instance {
    position: vec2f,
}
