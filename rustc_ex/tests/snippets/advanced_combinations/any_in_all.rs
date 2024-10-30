#[cfg(all(any(feature = "a", not(feature = "b")), feature = "c"))]
fn all() {

  #[cfg(any(feature = "d", feature = "e"))]
  fn any() {}

}
