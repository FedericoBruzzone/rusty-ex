#[cfg(feature = "struct-declaration-1")]
struct X {
    x: i32,
}

#[cfg(feature = "main")]
fn main() {

    #[cfg(feature = "struct-declaration-2")]
    struct Y {
        y: i32,
    }
}
