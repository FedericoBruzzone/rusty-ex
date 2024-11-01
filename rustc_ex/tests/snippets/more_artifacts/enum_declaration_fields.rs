enum X {
    #[cfg(feature = "enum-1-fields-1")]
    A,
    #[cfg(feature = "enum-1-fields-2")]
    B(String)
}

#[cfg(feature = "main")]
fn main() {

    enum Y {
        #[cfg(feature = "enum-2-fields-1")]
        A,
        #[cfg(feature = "enum-2-fields-2")]
        B(String)
    }
}
