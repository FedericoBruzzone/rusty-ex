trait Add {
    fn add(&self, b: i32) -> i32;
}

impl Add for i32 {
    fn add(&self, b: i32) -> i32 {
        self + b
    }
}

#[cfg(feature = "main")]
fn main() {

    trait Sub {
        fn sub(a: i32, b: i32) -> i32;
    }

    impl Sub for i32 {
        fn sub(a: i32, b: i32) -> i32 {
            a - b
        }
    }

    #[cfg(feature = "method-call-1")]
    i32::sub(1, 2);

    #[cfg(feature = "method-call-2")]
    1.add(2);

    #[cfg(feature = "method-call-3")]
    let _ = 1.add(2);

    #[cfg(feature = "method-call-4")]
    1.add(i32::sub(1, 2));
}
