#[cfg(feature = "type-alias-1")]
type Num = i32;

#[cfg(feature = "main")]
fn main() {

    #[cfg(feature = "type-alias-2")]
    type Point = (u8, u8);
}
