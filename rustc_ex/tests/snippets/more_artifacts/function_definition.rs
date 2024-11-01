#[cfg(feature = "function-definition-1")]
fn sub(a: i32, b: i32) -> i32 {
    a - b
}

#[cfg(feature = "main")]
fn main() {

    #[cfg(feature = "function-definition-2")]
    fn add(a: i32, b: i32) -> i32 {
        a + b
    }
}
