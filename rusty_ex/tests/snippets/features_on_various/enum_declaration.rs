#[cfg(feature = "enum-declaration-1")]
enum X {
    A,
    B(String)
}

#[cfg(feature = "main")]
fn main() {

    #[cfg(feature = "enum-declaration-2")]
    enum Y {
        A,
        B(String)
    }
}
