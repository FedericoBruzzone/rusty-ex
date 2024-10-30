#[cfg(all(any(feature = "a", not(feature = "b")), feature = "c"))]
fn all() {

  #[cfg(feature = "d")]
  fn one() {}

}
