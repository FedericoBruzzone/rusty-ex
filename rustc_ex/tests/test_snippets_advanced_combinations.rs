mod utils;

// use pretty_assertions::assert_eq;
use utils::run_with_cargo_bin_and_snippet;

const FOLDER: &str = "tests/snippets/advanced_combinations";

// =============================================

// Advanced tests for the different combinations of cfg attributes
//
//          all(any(one not) one) any(all(one not) one)
// one             x                      x
// not             x                      x
// any             x                      x
// all             x                      x

// =============================================
// ==================== ALL ====================
// =============================================

#[test]
#[ignore]
fn test_one_in_all() -> Result<(), String> {
    unimplemented!()
}

#[test]
#[ignore]
fn test_not_in_all() -> Result<(), String> {
    unimplemented!()
}

#[test]
#[ignore]
fn test_any_in_all() -> Result<(), String> {
    unimplemented!()
}

#[test]
#[ignore]
fn test_all_in_all() -> Result<(), String> {
    unimplemented!()
}

// =============================================
// ==================== ANY ====================
// =============================================

#[test]
#[ignore]
fn test_one_in_any() -> Result<(), String> {
    unimplemented!()
}

#[test]
#[ignore]
fn test_not_in_any() -> Result<(), String> {
    unimplemented!()
}

#[test]
#[ignore]
fn test_any_in_any() -> Result<(), String> {
    unimplemented!()
}

#[test]
#[ignore]
fn test_all_in_any() -> Result<(), String> {
    unimplemented!()
}
