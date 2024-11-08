trait Add {
    fn add(&self, b: i32) -> i32;
}

impl Add for i32 {
    #[cfg(feature = "method-definition")]
    fn add(&self, b: i32) -> i32 {
        self + b
    }
}

#[cfg(feature = "main")]
fn main() {

    trait Sub {
        fn sub(a: i32, b: i32) -> i32;
    }

    #[cfg(feature = "static-method-definition")]
    impl Sub for i32 {
        fn sub(a: i32, b: i32) -> i32 {
            a - b
        }
    }

}
