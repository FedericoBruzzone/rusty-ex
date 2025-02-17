fn bar() {}

#[cfg(all(feature = "f1", feature = "f2"))]
fn foo() {
    #[cfg(any(feature = "f2", feature = "f3", all(feature = "f2", feature = "f3")))]
    bar("hello");

    #[cfg(not(feature = "f3"))]
    bar();
}

#[cfg(any(feature = "f1", feature = "f2"))]
fn foo() {}

fn main() {
    foo()
}
