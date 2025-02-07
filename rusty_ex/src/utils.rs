macro_rules! bx {
    ($e:expr) => {
        Box::new($e)
    };
}

pub(crate) use bx;
