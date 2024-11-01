#[cfg(feature = "main")]
fn main() {

    let x = false;

    #[cfg(feature = "match-1")]
    match x {
        true => {

            #[cfg(feature = "match-2")]
            match !x {
                true => (),
                false => (),
            }

        },
        false => (),
    }

}
