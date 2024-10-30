#[cfg(all(any(feature = "a", not(feature = "b")), feature = "c"))]
fn all() {

  #[cfg(not(feature = "d"))]
  fn not() {}

}
