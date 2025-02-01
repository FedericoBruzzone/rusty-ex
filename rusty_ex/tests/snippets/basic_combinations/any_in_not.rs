#[cfg(not(feature = "a"))]
fn not_a() {

    #[cfg(any(feature = "b", feature = "c"))]
    fn any_b_c() {}

}
