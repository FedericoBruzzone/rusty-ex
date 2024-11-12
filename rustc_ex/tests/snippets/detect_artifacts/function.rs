fn example1() {}

fn example2<T>() {}

fn example3<T: Debug>() {}

fn example4<T: Debug>(a: T) {}

fn example5<T: Debug>(a: T) -> T {}

fn example6<T: Debug>(a: T) -> T where T: Clone {}

extern "ABI" fn example7() {}

const fn example8() {}

async fn example9() {}

async unsafe fn example10() { }

fn example11(
    #[cfg(feature = "a")] slice: &[u16],
    #[cfg(not(feature = "b"))] slice: &[u8],
) {
    slice.len()
}
