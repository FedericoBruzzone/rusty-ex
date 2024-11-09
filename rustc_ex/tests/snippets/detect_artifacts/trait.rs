trait Example1 {}

trait Example2 {
    const A: u32;
    type B;
    const C: u32 = 0;
    fn example();
    fn example(&self) {}
}

trait Example3<T> {
    fn example<T>(&self) -> T;
}

trait Example4 { fn area(&self) -> f64; }
trait Example5 : Example4 { fn radius(&self) -> f64; }

trait Example6 { fn area(&self) -> f64; }
trait Example7 where Self: Example6 { fn radius(&self) -> f64; }
