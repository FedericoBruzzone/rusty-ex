#[cfg(feature = "a")]
fn a() {

    #[cfg(any(feature = "b", feature = "c"))]
    fn any_b_c() {}

}
