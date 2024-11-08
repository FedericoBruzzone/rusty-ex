fn sub(a: i32, b: i32) -> i32 {
    a - b
}

#[cfg(feature = "main")]
fn main() {

    fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    #[cfg(feature = "function-call-1")]
    add(1, 2);

    #[cfg(feature = "function-call-2")]
    sub(1, 2);

    #[cfg(feature = "function-call-3")]
    let _ = add(1, 2);

    #[cfg(feature = "function-call-4")]
    sub(add(1, 2), add(3, 4));
}
