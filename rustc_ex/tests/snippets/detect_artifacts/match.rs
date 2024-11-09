fn main() {

    match 10 {
        1 => (),
        2 => (),
        _ => (),
    }

    match 5 {};

    match 5 {
        1 if true => (),
        1 => (),
        2 if false => (),
        _ => (),
    }

}
