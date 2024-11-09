fn main() {
    123;
    123i32;
    123u32;
    123_u32;
    0xff;
    0xff_u8;
    0o70;
    0o70_i16;
    0b1111_1111_1001_0000;
    0b1111_1111_1001_0000i64;
    0usize;

    123.0f64;
    0.1f64;
    0.1f32;
    12E+99_f64;
    5f32;
    2.;

    c"foo";
    cr"foo";
    c"\"foo\"";
    cr#""foo""#;
    c"foo #\"# bar";
    cr##"foo #"# bar"##;
    c"\x52";
    c"R";
    cr"R";
    c"\\x52";
    cr"\x52";
    c"æ";
    c"\u{00E6}";
    c"\xC3\xA6";

    b"foo";
    br"foo";
    b"\"foo\"";
    br#""foo""#;
    b"foo #\"# bar";
    br##"foo #"# bar"##;
    b"\x52";
    b"R";
    br"R";
    b"\\x52";
    br"\x52";

    b'R';
    b'\'';
    b'\x52';
    b'\xA0';

    "foo";
    r"foo";
    "\"foo\"";
    r#""foo""#;

    "foo #\"# bar";
    r##"foo #"# bar"##;

    "\x52";
    "R";
    r"R";
    "\\x52";
    r"\x52";

    'R';
    '\'';
    '\x52';
    '\u{00E6}';
}
