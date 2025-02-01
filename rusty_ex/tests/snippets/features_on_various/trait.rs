#[cfg(feature = "trait-1")]
trait Add {
    fn add(a: i32, b: i32) -> i32;
}

#[cfg(feature = "main")]
fn main() {

    #[cfg(feature = "trait-2")]
    trait Sub {
        fn sub(&self, a: i32, b: i32) -> i32;
    }

}
