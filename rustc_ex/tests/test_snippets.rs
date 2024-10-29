mod utils;

// use pretty_assertions::assert_eq;
use utils::run_with_cargo_bin_and_snippet;

const BASIC_COMBINATIONS_FOLDER: &str = "tests/snippets/basic_combinations";


#[test]
fn test_snippets_example() -> Result<(), String> {
    let snippet = r#"
#[cfg(feature = "a")]
fn a() {}

#[cfg(all(feature = "b", feature = "c"))]
fn all_b_c() {}
"#;
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

    assert!(output.contains("a"));
    assert!(output.contains("b"));
    assert!(output.contains("c"));

    Ok(())
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
    let snippet =
        &std::fs::read_to_string(format!("{BASIC_COMBINATIONS_FOLDER}/one_in_one.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"a\"]"));
    assert!(output.contains("2 [ label=\"b\"]"));
    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_one_in_not() -> Result<(), String> {
    let snippet =
        &std::fs::read_to_string(format!("{BASIC_COMBINATIONS_FOLDER}/one_in_not.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"!a\"]"));
    assert!(output.contains("2 [ label=\"b\"]"));
    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_one_in_any() -> Result<(), String> {
    let snippet =
        &std::fs::read_to_string(format!("{BASIC_COMBINATIONS_FOLDER}/one_in_any.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"a\"]"));
    assert!(output.contains("2 [ label=\"b\"]"));
    assert!(output.contains("3 [ label=\"c\"]"));
    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_one_in_all() -> Result<(), String> {
    let snippet =
        &std::fs::read_to_string(format!("{BASIC_COMBINATIONS_FOLDER}/one_in_all.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"a\"]"));
    assert!(output.contains("2 [ label=\"b\"]"));
    assert!(output.contains("3 [ label=\"c\"]"));
    assert!(output.contains("1 -> 0 [ label=\"0.50\"]"));
    assert!(output.contains("2 -> 0 [ label=\"0.50\"]"));
    assert!(output.contains("3 -> 1 [ label=\"2.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"2.00\"]"));

    Ok(())
}

// =============================================
// ==================== NOT ====================
// =============================================

#[test]
fn test_not_in_one() -> Result<(), String> {
    let snippet =
        &std::fs::read_to_string(format!("{BASIC_COMBINATIONS_FOLDER}/not_in_one.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"a\"]"));
    assert!(output.contains("2 [ label=\"!b\"]"));
    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_not_in_not() -> Result<(), String> {
    let snippet =
        &std::fs::read_to_string(format!("{BASIC_COMBINATIONS_FOLDER}/not_in_not.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"!a\"]"));
    assert!(output.contains("2 [ label=\"!b\"]"));
    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_not_in_any() -> Result<(), String> {
    let snippet =
        &std::fs::read_to_string(format!("{BASIC_COMBINATIONS_FOLDER}/not_in_any.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"a\"]"));
    assert!(output.contains("2 [ label=\"b\"]"));
    assert!(output.contains("3 [ label=\"!c\"]"));
    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_not_in_all() -> Result<(), String> {
    let snippet =
        &std::fs::read_to_string(format!("{BASIC_COMBINATIONS_FOLDER}/not_in_all.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"a\"]"));
    assert!(output.contains("2 [ label=\"b\"]"));
    assert!(output.contains("3 [ label=\"!c\"]"));
    assert!(output.contains("1 -> 0 [ label=\"0.50\"]"));
    assert!(output.contains("2 -> 0 [ label=\"0.50\"]"));
    assert!(output.contains("3 -> 1 [ label=\"2.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"2.00\"]"));

    Ok(())
}

// =============================================
// ==================== ALL ====================
// =============================================

#[test]
fn test_all_in_one() -> Result<(), String> {
    let snippet =
        &std::fs::read_to_string(format!("{BASIC_COMBINATIONS_FOLDER}/all_in_one.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"a\"]"));
    assert!(output.contains("2 [ label=\"b\"]"));
    assert!(output.contains("3 [ label=\"c\"]"));
    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"0.50\"]"));
    assert!(output.contains("3 -> 1 [ label=\"0.50\"]"));

    Ok(())
}

#[test]
fn test_all_in_not() -> Result<(), String> {
    let snippet =
        &std::fs::read_to_string(format!("{BASIC_COMBINATIONS_FOLDER}/all_in_not.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"!a\"]"));
    assert!(output.contains("2 [ label=\"b\"]"));
    assert!(output.contains("3 [ label=\"c\"]"));
    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"0.50\"]"));
    assert!(output.contains("3 -> 1 [ label=\"0.50\"]"));

    Ok(())
}

#[test]
fn test_all_in_any() -> Result<(), String> {
    let snippet =
        &std::fs::read_to_string(format!("{BASIC_COMBINATIONS_FOLDER}/all_in_any.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"a\"]"));
    assert!(output.contains("2 [ label=\"b\"]"));
    assert!(output.contains("3 [ label=\"c\"]"));
    assert!(output.contains("4 [ label=\"d\"]"));
    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 1 [ label=\"0.50\"]"));
    assert!(output.contains("3 -> 2 [ label=\"0.50\"]"));
    assert!(output.contains("4 -> 1 [ label=\"0.50\"]"));
    assert!(output.contains("4 -> 2 [ label=\"0.50\"]"));

    Ok(())
}

#[test]
fn test_all_in_all() -> Result<(), String> {
    let snippet =
        &std::fs::read_to_string(format!("{BASIC_COMBINATIONS_FOLDER}/all_in_all.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"a\"]"));
    assert!(output.contains("2 [ label=\"b\"]"));
    assert!(output.contains("3 [ label=\"c\"]"));
    assert!(output.contains("4 [ label=\"d\"]"));
    assert!(output.contains("1 -> 0 [ label=\"0.50\"]"));
    assert!(output.contains("2 -> 0 [ label=\"0.50\"]"));
    assert!(output.contains("3 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

// =============================================
// ==================== ANY ====================
// =============================================

#[test]
fn test_any_in_one() -> Result<(), String> {
    let snippet =
        &std::fs::read_to_string(format!("{BASIC_COMBINATIONS_FOLDER}/any_in_one.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"a\"]"));
    assert!(output.contains("2 [ label=\"b\"]"));
    assert!(output.contains("3 [ label=\"c\"]"));
    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 1 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_any_in_not() -> Result<(), String> {
    let snippet =
        &std::fs::read_to_string(format!("{BASIC_COMBINATIONS_FOLDER}/any_in_not.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"!a\"]"));
    assert!(output.contains("2 [ label=\"b\"]"));
    assert!(output.contains("3 [ label=\"c\"]"));
    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 1 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_any_in_any() -> Result<(), String> {
    let snippet =
        &std::fs::read_to_string(format!("{BASIC_COMBINATIONS_FOLDER}/any_in_any.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"a\"]"));
    assert!(output.contains("2 [ label=\"b\"]"));
    assert!(output.contains("3 [ label=\"c\"]"));
    assert!(output.contains("4 [ label=\"d\"]"));
    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_any_in_all() -> Result<(), String> {
    let snippet =
        &std::fs::read_to_string(format!("{BASIC_COMBINATIONS_FOLDER}/any_in_all.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"a\"]"));
    assert!(output.contains("2 [ label=\"b\"]"));
    assert!(output.contains("3 [ label=\"c\"]"));
    assert!(output.contains("4 [ label=\"d\"]"));
    assert!(output.contains("1 -> 0 [ label=\"0.50\"]"));
    assert!(output.contains("2 -> 0 [ label=\"0.50\"]"));
    assert!(output.contains("3 -> 1 [ label=\"2.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"2.00\"]"));
    assert!(output.contains("4 -> 1 [ label=\"2.00\"]"));
    assert!(output.contains("4 -> 2 [ label=\"2.00\"]"));

    Ok(())
}

// =============================================

// Advanced tests for the different combinations of cfg attributes
//
//          all(any(one not) one) any(all(one not) one)
// one
// not
// any
// all
