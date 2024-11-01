trait Add {
    fn add(a: i32, b: i32) -> i32;
}

#[cfg(feature = "implementation-1")]
impl Add for i32 {
    fn add(a: i32, b: i32) -> i32 {
        a + b
    }
}

#[cfg(feature = "main")]
fn main() {

    trait Sub {
        fn sub(&self, a: i32, b: i32) -> i32;
    }

    #[cfg(feature = "implementation-2")]
    impl Sub for i32 {
        fn sub(&self, a: i32, b: i32) -> i32 {
            a - b
        }
    }

}
