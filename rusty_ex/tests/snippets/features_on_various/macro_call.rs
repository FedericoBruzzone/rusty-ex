#[cfg(feature = "main")]
fn main() {

    #[cfg(feature = "macro-call-1")]
    println!("ciao");

    #[cfg(feature = "macro-call-2")]
    vec![1, 2, 3];
}
