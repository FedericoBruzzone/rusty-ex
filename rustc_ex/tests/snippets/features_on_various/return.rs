fn void1() {
    #[cfg(feature = "return-1")]
    return
}

#[cfg(feature = "main")]
fn main() {

    fn void2() {
        #[cfg(feature = "return-2")]
        return
    }

    #[cfg(feature = "return-3")]
    return
}
