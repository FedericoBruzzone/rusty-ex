#[cfg(feature = "aa")]
pub fn one() {}

#[cfg(any(feature = "bb", all(feature = "bb")))]
pub fn two() {}
