enum X {
    A,
    B,
    C,
    D
}

#[cfg(feature = "main")]
fn main() {

    let x = X::A;

    match x {
        #[cfg(feature = "match-branch-1")]
        X::A => (),
        X::B => (),
        X::C => (),
        #[cfg(feature = "match-branch-2")]
        X::D => (),
        _ => (),
    }

}
