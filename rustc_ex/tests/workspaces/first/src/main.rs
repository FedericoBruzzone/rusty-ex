#[cfg(feature = "cc")]
fn three() {

    #[cfg(feature = "aa")]
    lib1::one();

    println!("Hello, world!");

    #[cfg(feature = "dd")]
    fn four() {

        #[cfg(feature = "ee")]
        fn five() {}

        #[cfg(not(feature = "ff"))]
        fn six() {}

    }
}

fn main() {}
