#[cfg(feature = "aa")]
fn one() {

    #[cfg(any(feature = "bb", feature = "cc"))]
    fn two() {}

}

#[cfg(all(feature = "ee", not(feature = "ff")))]
fn three() {

    #[cfg(feature = "dd")]
    fn four() {}

}
