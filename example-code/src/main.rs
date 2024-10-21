// #[cfg(all(feature = "aa", feature = "bb"))]
#[cfg(feature = "aa")]
fn one() {}

#[cfg(any(feature = "aa", all(feature = "bb", not(feature = "cc"))))]
fn two() {}

#[cfg(feature = "bb")]
fn three() {

    #[cfg(feature = "cc")]
    fn four() {

        #[cfg(feature = "bb")]
        fn five() {}

    }
}

fn main() {}
