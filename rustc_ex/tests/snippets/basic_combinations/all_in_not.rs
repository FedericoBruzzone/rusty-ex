#[cfg(not(feature = "a"))]
fn not_a() {

    #[cfg(all(feature = "b", feature = "c"))]
    fn all_b_c() {}

}
