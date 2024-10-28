#[cfg(feature = "aa")]
fn one() {

    #[cfg(any(feature = "bb", feature = "cc"))]
    fn two() {}

}

#[cfg(feature = "dd")]
fn three() {

    #[cfg(all(feature = "ee", feature = "ff"))]
    fn four() {}

}
