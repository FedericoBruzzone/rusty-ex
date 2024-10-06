mod mod1;

#[cfg(feature = "f1")]
fn ciao() {
    println!("Ciao");
}

#[cfg(feature = "f2")]
fn ciao() {
    println!("Ciao");
}

fn main() {
    ciao();
    mod1::hello();
}
