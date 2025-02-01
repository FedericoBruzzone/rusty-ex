#[cfg(all(feature = "a", feature = "b"))]
fn a_b() {

    #[cfg(feature = "c")]
    fn c() {}

}
