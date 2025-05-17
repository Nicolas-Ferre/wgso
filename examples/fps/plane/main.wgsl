#init init_plane()
#draw plane<plane.vertices, plane.instance>(camera=camera)

#import _.std.vertex

struct Plane {
    vertices: array<Vertex, 6>,
    instance: PlaneInstance,
}

struct PlaneInstance {
    size: vec2f,
    color: vec4f,
}
