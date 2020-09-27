pub fn cmp_float(a: f32, b: f32) -> bool {
    (a - b).abs() <= f32::EPSILON * a.abs().max(b.abs())
}
