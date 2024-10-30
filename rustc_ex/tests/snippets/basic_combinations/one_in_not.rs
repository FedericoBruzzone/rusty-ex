#[cfg(not(feature = "a"))]
fn not_a() {

    #[cfg(feature = "b")]
    fn b() {}

}
