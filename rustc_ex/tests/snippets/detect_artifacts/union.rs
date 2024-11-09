union Example1 {
    a: u32,
    b: f32,
}

union Example2 {}

union Example3 {
    #[cfg(feature = "a")]
    a: u32,
    b: f32,
}
