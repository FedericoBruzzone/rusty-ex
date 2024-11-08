#[cfg(feature = "main")]
fn main() {

    let vec = vec![1, 2, 3, 4, 5];

    #[cfg(feature = "for")]
    for i in vec {

        #[cfg(feature = "while")]
        while i < 3 {

            #[cfg(feature = "loop-1")]
            loop {}

        }
    }

    #[cfg(feature = "loop-2")]
    loop {}

}
