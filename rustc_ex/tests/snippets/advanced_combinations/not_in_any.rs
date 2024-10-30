#[cfg(any(all(feature = "a", not(feature = "b")), feature = "c"))]
fn any() {

  #[cfg(not(feature = "d"))]
  fn not() {}

}
