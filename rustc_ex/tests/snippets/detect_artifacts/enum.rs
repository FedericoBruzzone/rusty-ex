enum Example1 {}

enum Example2 {
    A,
    B,
}

enum Example3 {
    A,
    B(u32),
    C{a: u32, b: u32},
}

enum Example4 {
    A = 1,
    B = 2,
}

enum Example5 {
    A = 1,
    B = 2,
    #[cfg(feature = "a")]
    C = 3,
    #[cfg(not(windows))]
    D = 4,
}

enum Example6 {
    A,
    B = 3,
    C,
}
