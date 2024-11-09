struct Example1;

struct Example2 {
    a: u32,
    b: u32,
}

struct Example3(u32);

struct Example4 {
    #[cfg(feature = "a")]
    a: u32,
    #[cfg(not(windows))]
    b: u32,
}
