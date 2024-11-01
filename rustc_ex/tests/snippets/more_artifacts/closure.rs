#[cfg(feature = "main")]
fn main() {

  #[cfg(feature = "closure-1")]
  |a: i32, b: i32| -> i32 {
    a + b
  };

  #[cfg(feature = "closure-2")]
  let add = |a: i32, b: i32| -> i32 {
    a + b
  };

}
