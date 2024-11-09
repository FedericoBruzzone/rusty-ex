// global block 1

fn main() { // block 2

    // block 3
    { }

    // block 4
    {
        // block 5
        { }
    };

}

fn b() { // block 6

    let five: i32 = { // block 7
        example();
        5
    };

    loop { // block 8
        async move { /* block 9 */ };
    };

    // item block 10
    const SIX: i32 = { /* block 11 */ };

    unsafe { /* block 12 */ };

    'a: { /* block 13 */ };

}
