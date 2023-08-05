// utility function to compare enum variants
pub fn variants_equal<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}
