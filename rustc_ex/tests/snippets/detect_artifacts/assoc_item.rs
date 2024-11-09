struct Struct;
impl Struct {
    fn example1() -> Struct {
        Struct;
    }

    fn example2(&self) {}

    fn example3(&mut self) {}

    fn example4<'a>(self: &mut &'a Arc<Rc<Box<Alias>>>) {}
}

trait Trait {
    fn example5(n: i32) -> Self;
}

impl Trait for f64 {
    fn example6(n: i32) -> f64 {
        n as f64
    }
}

trait Type {
    type Example1;
    type Example2 = f64;
    type Example3: Trait + 'static;
    type Example4 where Self: Sized;

    const EXAMPLE1: i32 = 0;
    const EXAMPLE2: () = ();
}
