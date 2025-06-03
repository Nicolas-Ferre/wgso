/// Vertex type.
#mod type

/// Vertex data.
struct Vertex {
    position: vec3f,
    normal: vec3f,
}

/// Vertex definition of common models.
#mod model
#import ~.type

/// Returns the vertices of a rectangle.
///
/// Vertices are defined on X and Y axis between -0.5 and 0.5.
/// Z is 0.
fn rectangle_vertices() -> array<Vertex, 6> {
    return array(
        Vertex(vec3f(-.5, -.5, 0), vec3f(0, 0, 1)),
        Vertex(vec3f(.5, -.5, 0), vec3f(0, 0, 1)),
        Vertex(vec3f(-.5, .5, 0), vec3f(0, 0, 1)),
        Vertex(vec3f(.5, .5, 0), vec3f(0, 0, 1)),
        Vertex(vec3f(-.5, .5, 0), vec3f(0, 0, 1)),
        Vertex(vec3f(.5, -.5, 0), vec3f(0, 0, 1)),
    );
}

/// Returns the vertices of a rectangle.
///
/// Vertices are defined on each axis between -0.5 and 0.5.
fn cube_vertices() -> array<Vertex, 36> {
    return array(
        // +z face
        Vertex(vec3f(-.5, -.5, .5), vec3f(0, 0, 1)),
        Vertex(vec3f(.5, .5, .5), vec3f(0, 0, 1)),
        Vertex(vec3f(.5, -.5, .5), vec3f(0, 0, 1)),
        Vertex(vec3f(-.5, -.5, .5), vec3f(0, 0, 1)),
        Vertex(vec3f(-.5, .5, .5), vec3f(0, 0, 1)),
        Vertex(vec3f(.5, .5, .5), vec3f(0, 0, 1)),
        // -z face
        Vertex(vec3f(.5, -.5, -.5), vec3f(0, 0, -1)),
        Vertex(vec3f(-.5, .5, -.5), vec3f(0, 0, -1)),
        Vertex(vec3f(-.5, -.5, -.5), vec3f(0, 0, -1)),
        Vertex(vec3f(.5, -.5, -.5), vec3f(0, 0, -1)),
        Vertex(vec3f(.5, .5, -.5), vec3f(0, 0, -1)),
        Vertex(vec3f(-.5, .5, -.5), vec3f(0, 0, -1)),
        // +x face
        Vertex(vec3f(.5, -.5, .5), vec3f(1, 0, 0)),
        Vertex(vec3f(.5, .5, -.5), vec3f(1, 0, 0)),
        Vertex(vec3f(.5, -.5, -.5), vec3f(1, 0, 0)),
        Vertex(vec3f(.5, -.5, .5), vec3f(1, 0, 0)),
        Vertex(vec3f(.5, .5, .5), vec3f(1, 0, 0)),
        Vertex(vec3f(.5, .5, -.5), vec3f(1, 0, 0)),
        // -x face
        Vertex(vec3f(-.5, -.5, -.5), vec3f(-1, 0, 0)),
        Vertex(vec3f(-.5, .5, .5), vec3f(-1, 0, 0)),
        Vertex(vec3f(-.5, -.5, .5), vec3f(-1, 0, 0)),
        Vertex(vec3f(-.5, -.5, -.5), vec3f(-1, 0, 0)),
        Vertex(vec3f(-.5, .5, -.5), vec3f(-1, 0, 0)),
        Vertex(vec3f(-.5, .5, .5), vec3f(-1, 0, 0)),
        // +y face
        Vertex(vec3f(-.5, .5, .5), vec3f(0, 1, 0)),
        Vertex(vec3f(.5, .5, -.5), vec3f(0, 1, 0)),
        Vertex(vec3f(.5, .5, .5), vec3f(0, 1, 0)),
        Vertex(vec3f(-.5, .5, .5), vec3f(0, 1, 0)),
        Vertex(vec3f(-.5, .5, -.5), vec3f(0, 1, 0)),
        Vertex(vec3f(.5, .5, -.5), vec3f(0, 1, 0)),
        // -y face
        Vertex(vec3f(-.5, -.5, -.5), vec3f(0, -1, 0)),
        Vertex(vec3f(.5, -.5, .5), vec3f(0, -1, 0)),
        Vertex(vec3f(.5, -.5, -.5), vec3f(0, -1, 0)),
        Vertex(vec3f(-.5, -.5, -.5), vec3f(0, -1, 0)),
        Vertex(vec3f(-.5, -.5, .5), vec3f(0, -1, 0)),
        Vertex(vec3f(.5, -.5, .5), vec3f(0, -1, 0)),
    );
}
