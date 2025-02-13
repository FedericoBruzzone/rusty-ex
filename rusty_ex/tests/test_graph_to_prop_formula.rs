#![feature(rustc_private)]

mod utils;

use pretty_assertions::assert_eq;
use rusty_ex::configs::prop_formula::{ConversionMethod, ToPropFormula};
use rusty_ex::types::{FeaturesGraph, SimpleSerialization};
use utils::{bx, run_with_cargo_bin_and_snippet};

const FOLDER: &str = "tests/snippets/basic_combinations";

fn get_feature_graph(file: &str) -> Result<FeaturesGraph, String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/{file}")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-serialized-graphs"])?;
    let deserialized_graph: SimpleSerialization = serde_json::from_str(&output).unwrap();
    Ok(deserialized_graph.features_graph)
}

// =============================================

// Basic tests for the different combinations of cfg attributes
//
//     one not any all
// one  x   x   x   x
// not  x   x   x   x
// any  x   x   x   x
// all  x   x   x   x

// =============================================
// ==================== ONE ====================
// =============================================

#[test]
fn test_one_in_one() -> Result<(), String> {
    use rusty_ex::configs::prop_formula::PropFormula::*;

    let feat_graph = get_feature_graph("one_in_one.rs")?;
    // feat_graph.print_dot();
    let prop_formula = feat_graph.to_prop_formula(ConversionMethod::Naive);
    let output = And(vec![
        Var("__GLOBAL__".to_string()),
        Var("__DUMMY__".to_string()),
        Var("a".to_string()),
        Var("b".to_string()),
    ]);

    assert_eq!(prop_formula, output);

    Ok(())
}

#[test]
fn test_one_in_not() -> Result<(), String> {
    use rusty_ex::configs::prop_formula::PropFormula::*;

    let feat_graph = get_feature_graph("one_in_not.rs")?;
    // feat_graph.print_dot();
    let prop_formula = feat_graph.to_prop_formula(ConversionMethod::Naive);

    let output = And(vec![
        Var("__GLOBAL__".to_string()),
        Var("__DUMMY__".to_string()),
        Not(bx!(Var("a".to_string(),))),
        Var("b".to_string()),
    ]);

    assert_eq!(prop_formula, output);

    Ok(())
}

#[test]
fn test_one_in_any() -> Result<(), String> {
    use rusty_ex::configs::prop_formula::PropFormula::*;

    let feat_graph = get_feature_graph("one_in_any.rs")?;
    // feat_graph.print_dot();
    let prop_formula = feat_graph.to_prop_formula(ConversionMethod::Naive);

    let output = And(vec![
        Var("__GLOBAL__".to_string()),
        Var("__DUMMY__".to_string()),
        Or(vec![Var("a".to_string()), Var("b".to_string())]),
        Or(vec![Var("a".to_string()), Var("b".to_string())]),
        Var("c".to_string()),
    ]);

    assert_eq!(prop_formula, output);

    Ok(())
}

#[test]
fn test_one_in_all() -> Result<(), String> {
    use rusty_ex::configs::prop_formula::PropFormula::*;

    let feat_graph = get_feature_graph("one_in_all.rs")?;
    // feat_graph.print_dot();
    let prop_formula = feat_graph.to_prop_formula(ConversionMethod::Naive);

    let output = And(vec![
        Var("__GLOBAL__".to_string()),
        Var("__DUMMY__".to_string()),
        And(vec![Var("a".to_string()), Var("b".to_string())]),
        And(vec![Var("a".to_string()), Var("b".to_string())]),
        Var("c".to_string()),
    ]);

    assert_eq!(prop_formula, output);

    Ok(())
}

// =============================================
// ==================== NOT ====================
// =============================================

#[test]
fn test_not_in_one() -> Result<(), String> {
    use rusty_ex::configs::prop_formula::PropFormula::*;

    let feat_graph = get_feature_graph("not_in_one.rs")?;
    // feat_graph.print_dot();
    let prop_formula = feat_graph.to_prop_formula(ConversionMethod::Naive);

    let output = And(vec![
        Var("__GLOBAL__".to_string()),
        Var("__DUMMY__".to_string()),
        Var("a".to_string()),
        Not(bx!(Var("b".to_string(),))),
    ]);

    assert_eq!(prop_formula, output);

    Ok(())
}

// #[test]
// fn test_not_in_not() -> Result<(), String> {}
//
// #[test]
// fn test_not_in_any() -> Result<(), String> {}
//
// #[test]
// fn test_not_in_all() -> Result<(), String> {}

// =============================================
// ==================== ALL ====================
// =============================================

#[test]
fn test_all_in_one() -> Result<(), String> {
    use rusty_ex::configs::prop_formula::PropFormula::*;

    let feat_graph = get_feature_graph("all_in_one.rs")?;
    // feat_graph.print_dot();
    let prop_formula = feat_graph.to_prop_formula(ConversionMethod::Naive);
    let output = And(vec![
        Var("__GLOBAL__".to_string()),
        Var("__DUMMY__".to_string()),
        Var("a".to_string()),
        And(vec![Var("b".to_string()), Var("c".to_string())]),
        And(vec![Var("b".to_string()), Var("c".to_string())]),
    ]);

    assert_eq!(prop_formula, output);

    Ok(())
}

// #[test]
// fn test_all_in_not() -> Result<(), String> {}
//
// #[test]
// fn test_all_in_any() -> Result<(), String> {}
//
// #[test]
// fn test_all_in_all() -> Result<(), String> {}

// =============================================
// ==================== ANY ====================
// =============================================

// #[test]
// fn test_any_in_one() -> Result<(), String> {}
//
// #[test]
// fn test_any_in_not() -> Result<(), String> {}
//
// #[test]
// fn test_any_in_any() -> Result<(), String> {}
//
// #[test]
// fn test_any_in_all() -> Result<(), String> {}
