#[cfg(feature = "aa")]
fn one() {}

#[cfg(any(feature = "bb", all(feature = "bb")))]
fn two() {}
