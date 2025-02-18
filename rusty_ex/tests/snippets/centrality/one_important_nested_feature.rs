fn bar() {
    let _ = 1;
    let _ = 1;
    let _ = 1;
    let _ = 1;
    let _ = 1;
    let _ = 1;
    let _ = 1;
    let _ = 1;
    let _ = 1;
    let _ = 1;
    let _ = 1;
    let _ = 1;
}

#[cfg(feature = "f1")]
fn foo() {
    #[cfg(feature = "f2")]
    bar();
}

fn main() {}