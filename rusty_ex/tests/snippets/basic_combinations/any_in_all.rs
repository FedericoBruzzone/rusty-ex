#[cfg(all(feature = "a", feature = "b"))]
fn a_b() {

    #[cfg(any(feature = "c", feature = "d"))]
    fn c_d() {}

}
