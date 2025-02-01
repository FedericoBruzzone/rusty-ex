mod lib1;

#[cfg(feature = "cc")]
fn three() {

    #[cfg(feature = "dd")]
    fn four() {

        #[cfg(feature = "ee")]
        fn five() {}

        #[cfg(not(feature = "ff"))]
        fn six() {}

    }
}
