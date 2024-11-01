#[cfg(feature = "macro-definition-1")]
macro_rules! vec {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
            temp_vec
        }
    };
}

#[cfg(feature = "main")]
fn main() {

    #[cfg(feature = "macro-definition-2")]
    macro_rules! pat {
        ($i:ident) => (Some($i))
    }

    #[cfg(feature = "macro-definition-3")]
    macro_rules! example {
        () => {};
    }
}
