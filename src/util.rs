use ggez::graphics::Color;

pub fn fast_wobble(time: f32) -> f32 {
    f32::sin(time * 6.0)
}

pub fn slow_wobble(time: f32) -> f32 {
    f32::sin(time * 2.0)
}

pub fn mix_color(from: Color, to: Color, amount: f32) -> Color {
    Color::new(
        from.r + (to.r - from.r) * amount,
        from.g + (to.g - from.g) * amount,
        from.b + (to.b - from.b) * amount,
        from.a + (to.a - from.a) * amount,
    )
}

pub fn rgb_to_grayscale(rgb: Color) -> Color {
    let gray = 0.2989 * rgb.r + 0.5870 * rgb.g + 0.1140 * rgb.b;
    Color::new(gray, gray, gray, rgb.a)
}

/// overshoots past 1.0 then settles
pub fn ease_out_back(t: f32) -> f32 {
    const C1: f32 = 1.70158;
    const C3: f32 = C1 + 1.0;
    let u = t - 1.0;
    1.0 + C3 * u * u * u + C1 * u * u
}
