#[cfg(feature = "main")]
fn main() {


    #[cfg(feature = "block-1")]
    {
        const A: i32 = 5;
        const B: i32 = 5;
    }

    #[cfg(feature = "block-2")]
    {
        const C: i32 = 5;

        #[cfg(feature = "block-3")]
        {
            const D: i32 = 5;
        }

        const E: i32 = 5;
    }
}
