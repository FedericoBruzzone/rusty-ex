#[cfg(any(feature = "a", feature = "b"))]
fn a_b() {

    #[cfg(not(feature = "c"))]
    fn not_c() {}

}
