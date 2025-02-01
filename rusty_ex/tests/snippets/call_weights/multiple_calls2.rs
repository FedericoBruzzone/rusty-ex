fn b() {
    c();
}

fn c() {
    d();
}

fn a() {
    b();
}

fn main() {

    fn e() {
        f();
    }

    a();
}

fn d() {
    e();
}

fn f() {
    1;
}
