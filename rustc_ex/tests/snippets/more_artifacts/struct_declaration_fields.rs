struct X {
    #[cfg(feature = "struct-1-fields-1")]
    x: i32,

    #[cfg(feature = "struct-1-fields-2")]
    y: i32,
}

#[cfg(feature = "main")]
fn main() {

    struct X {
        #[cfg(feature = "struct-2-fields-1")]
        x: i32,

        #[cfg(feature = "struct-2-fields-2")]
        y: i32,
    }
}
