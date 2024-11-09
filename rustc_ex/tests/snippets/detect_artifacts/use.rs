use example1;

fn main() {
    use example2;
}

use example as example3;
pub use example4;
use example::{example5, example6};
use example::{example::{example7, example8}, example9};
use crate::{example10, example11};
use example12::*;
use example13 as _;
use example14::{self, example15};
