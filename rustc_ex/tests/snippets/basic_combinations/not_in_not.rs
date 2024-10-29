#[cfg(not(feature = "a"))]
fn not_a() {

    #[cfg(not(feature = "b"))]
    fn not_b() {}

}
