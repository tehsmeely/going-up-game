/// Basic f32, does not worry at all bout all the float edge cases
pub fn lerp(a: f32, b: f32, s: f32) -> f32 {
    a + ((b - a) * s)
}
