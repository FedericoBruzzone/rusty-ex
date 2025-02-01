#[cfg(any(all(feature = "a", not(feature = "b")), feature = "c"))]
fn any() {

  #[cfg(feature = "d")]
  fn one() {}

}
