#[cfg(all(any(feature = "a", not(feature = "b")), feature = "c"))]
fn all() {

  #[cfg(all(feature = "d", feature = "e"))]
  fn all() {}

}
