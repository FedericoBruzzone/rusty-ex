mod lib1;

#[cfg(feature = "aa")]
struct X {
    #[cfg(feature = "bb")]
    x: i32,

    #[cfg(not(feature = "cc"))]
    y: i32,

    #[cfg(feature = "dd")]
    z: i32,
}

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

fn main() {
    #[cfg(feature = "aa")]
    let xx = X;
}
