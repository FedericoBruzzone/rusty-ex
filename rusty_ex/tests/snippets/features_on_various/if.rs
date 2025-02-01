#[cfg(feature = "main")]
fn main() {

    #[cfg(feature = "if-1")]
    if true {

        #[cfg(feature = "if-2")]
        if 1 == 2 {

        } else if 2 == 1 {

        } else {

        }

    } else {

    }

}
