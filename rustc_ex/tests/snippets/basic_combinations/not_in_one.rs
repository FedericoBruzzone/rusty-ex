#[cfg(feature = "a")]
fn a() {

    #[cfg(not(feature = "b"))]
    fn not_b() {}

}
