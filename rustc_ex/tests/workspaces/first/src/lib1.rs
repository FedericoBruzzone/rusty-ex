#[cfg(feature = "aa")]
pub fn one() {}

#[cfg(not(feature = "bb"))]
pub fn two() {}
