#[cfg(feature = "a")]
fn a() {

    #[cfg(all(feature = "b", feature = "c"))]
    fn all_b_c() {}
}
