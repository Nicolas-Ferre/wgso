/// Utilities to calculate collisions between two objects.
#mod collision

/// The result of a collision check.
struct Collision {
    /// Whether the two objects are colliding.
    is_colliding: bool,
    /// The penetration vector between the first and the second object.
    penetration: vec2f,
}

/// Check collision between two Axis-Aligned Bounding Boxes.
fn aabb_collision(position1: vec2f, size1: vec2f, position2: vec2f, size2: vec2f) -> Collision {
    let delta = position2 - position1;
    let overlap = (size1 + size2) / 2 - abs(delta);
    if any(overlap <= vec2f(0)) {
        // no collision
        return Collision(false, vec2f(0, 0));
    }
    let is_x_lower_than_y = overlap.x < overlap.y;
    let normal = sign(delta) * vec2f(f32(is_x_lower_than_y), f32(!is_x_lower_than_y));
    let penetration = abs(normal) * (position1 - position2) + normal * (size1 / 2 + size2 / 2);
    return Collision(true, penetration);
}
