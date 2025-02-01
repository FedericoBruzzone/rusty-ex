#[cfg(feature = "union-1")]
union X {
    a: u32,
    b: f32,
}

#[cfg(feature = "main")]
fn main() {

    #[cfg(feature = "union-2")]
    union Y {
        a: u32,
        b: f32,
    }

}
