mod lib1;

#[cfg(feature = "cc")]
fn three() {

    // // #[cfg(not(feature = "aa"))]
    // one();

    // println!("Hello, world!");

    #[cfg(feature = "dd")]
    fn four() {

        #[cfg(feature = "ee")]
        fn five() {}

    }
}

fn main() {}
