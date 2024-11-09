impl Example1 for u32 {}

impl Example2 {
    fn example() {}
}

impl<T> Example3<T> for Vec<T> {
    const A: u32 = 0;
}

impl<const N: usize> Example4 for ConstGenericStruct<N> {}

impl<T, U> Example5<U> for u32 where U: HasAssocType<Ty = T> {
    const A: u32 = 0;
    static B: u32 = 0;
}
