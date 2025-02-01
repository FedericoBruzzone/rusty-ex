#[cfg(any(all(feature = "a", not(feature = "b")), feature = "c"))]
fn any() {

  #[cfg(any(feature = "d", feature = "e"))]
  fn any() {}

}
