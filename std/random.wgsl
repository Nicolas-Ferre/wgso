//! Random number generators.

/// Generates a random `u32` value between `min` and `max`.
///
/// This function is based on [`random()`](random) function.
fn random_u32(seed: ptr<function, u32>, min: u32, max: u32) -> u32 {
    return random(seed) % max(abs(max - min), 1) + min(min, max);
}

/// Generates a random `i32` value between `min` and `max`.
///
/// This function is based on [`random()`](random) function.
fn random_i32(seed: ptr<function, u32>, min: i32, max: i32) -> i32 {
    return bitcast<i32>(random(seed)) % max(abs(max - min), 1) + min(min, max);
}

/// Generates a random `f32` value between `min` and `max`.
///
/// This function is based on [`random()`](random) function.
fn random_f32(seed: ptr<function, u32>, min: f32, max: f32) -> f32 {
    return f32(random(seed)) * abs(max - min) / f32(1 << 31) + min(min, max);
}

/// Generates a random `u32` value between 0 and 2^31 based on a `seed`.
///
/// The seed is modified in-place, and the new value can be used to generate another random value.
///
/// This function internally uses the
/// [LCG (Linear Congruential Genrator)](https://en.wikipedia.org/wiki/Linear_congruential_generator)
/// algorithm, which is fast but not cryptographically secure.
fn random(seed: ptr<function, u32>) -> u32 {
    *seed = (*seed * 1103515245 + 12345) % (1 << 31);
    return *seed;
}
