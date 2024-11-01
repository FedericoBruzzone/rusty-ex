#[cfg(feature = "main")]
fn main() {

    let mut x = 0;

    #[cfg(feature = "assignment")]
    x = 5;
}
