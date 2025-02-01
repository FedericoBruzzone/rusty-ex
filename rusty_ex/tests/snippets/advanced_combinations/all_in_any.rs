#[cfg(any(all(feature = "a", not(feature = "b")), feature = "c"))]
fn all() {

  #[cfg(all(feature = "d", feature = "e"))]
  fn any() {}

}
