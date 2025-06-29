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

#mod transform
#import ~.~.math.matrix
#import ~.~.math.quaternion

/// Returns a 2D scaling factor to ajust aspect ratio.
///
/// Vertex positions can be multiplied by the value returned by this function in order to fix the aspect ratio.
/// It ensures that the visible area maintains the correct aspect ratio while fitting within `min_visible_size`.
///
/// The visible area with size `min_visible_size` is centered in `vec2f(0, 0)`.
fn scale_factor(surface_size: vec2u, min_visible_size: vec2f) -> vec2f {
    let ratio = f32(surface_size.x) / f32(surface_size.y);
    return select(
        vec2f(1, ratio) / (min_visible_size.x / 2),
        vec2f(1 / ratio, 1) / (min_visible_size.y / 2),
        ratio > min_visible_size.x / min_visible_size.y
    );
}

/// Returns a projection matrix.
///
/// `fov` is in radians.
fn proj_mat(surface_size: vec2u, fov: f32, far: f32, near: f32) -> mat4x4f {
    let ratio = f32(surface_size.x) / f32(surface_size.y);
    let focal_length = 1 / (2 * tan(fov / 2));
    return transpose(mat4x4f(
        focal_length, 0, 0, 0,
        0, focal_length * ratio, 0, 0,
        0, 0, far / (far - near), -far * near / (far - near),
        0, 0, 1, 0,
    ));
}

/// Returns a model transformation matrix.
fn model_mat(position: vec3f, scale: vec3f, rotation: vec4f) -> mat4x4f {
    return translation_mat(position) * rotation_mat(rotation) * scale_mat(scale);
}

/// Returns a view transformation matrix.
fn view_mat(position: vec3f, rotation: vec4f) -> mat4x4f {
    return rotation_mat(quat_inverse(rotation)) * translation_mat(-position);
}
