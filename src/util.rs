use ggez::graphics::Color;

// utility function to compare enum variants
pub fn variants_equal<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

pub fn fast_wobble(time: f32) -> f32 {
    f32::sin(time * 6.0)
}

pub fn slow_wobble(time: f32) -> f32 {
    f32::sin(time * 2.0)
}

pub fn rgb_to_grayscale(rgb: Color) -> Color {
    let gray = 0.2989 * rgb.r + 0.5870 * rgb.g + 0.1140 * rgb.b;
    Color::new(gray, gray, gray, rgb.a)
}
