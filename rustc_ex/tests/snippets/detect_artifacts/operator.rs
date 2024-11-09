fn main() {

    // borrow
    &7;
    &mut [1,2,3];
    &&7;

    // raw borrow
    &raw const 7;
    &raw mut [1,2,3];

    // dereference
    *7;
    *[1,2,3];

    // question mark
    "123".parse::<i32>()?;

    // negation
    -7;
    !true;

    // arithmetic
    7 + 7;
    7 - 7;
    7 * 7;
    7 / 7;
    7 % 7;
    7 & 7;
    7 | 7;
    7 ^ 7;
    7 << 7;
    7 >> 7;

    // comparison
    7 == 7;
    7 != 7;
    7 < 7;
    7 > 7;
    7 <= 7;
    7 >= 7;

    // lazy boolean
    true && true;
    true || true;

    // type cast
    7 as i32;

    // assignment
    let a;
    a = 7;

    let (b, c);
    b = c = 5;

    let (d, e);
    (d, e) = (5, 6);

    // compound assignment
    5 += 5;
    5 -= 5;
    5 *= 5;
    5 /= 5;
    5 %= 5;
    5 &= 5;
    5 |= 5;
    5 ^= 5;
    5 <<= 5;
    5 >>= 5;

}
