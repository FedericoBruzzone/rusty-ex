#[cfg(feature = "constant-1")]
const Y: i32 = 5;

#[cfg(feature = "main")]
fn main() {

    #[cfg(feature = "constant-2")]
    const X: i32 = 5;
}
