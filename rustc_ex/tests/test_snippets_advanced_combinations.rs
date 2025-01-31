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
fn test_one_in_all() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/one_in_all.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__"));
    assert!(output.contains("2 [ label=\"i2: a"));
    assert!(output.contains("3 [ label=\"i3: !b"));
    assert!(output.contains("4 [ label=\"i4: c"));
    assert!(output.contains("5 [ label=\"i5: d"));
    assert!(output.contains("2 -> 0 [ label=\"0.50"));
    assert!(output.contains("3 -> 0 [ label=\"0.50"));
    assert!(output.contains("4 -> 0 [ label=\"0.50"));
    assert!(output.contains("5 -> 2 [ label=\"1.00"));
    assert!(output.contains("5 -> 3 [ label=\"1.00"));
    assert!(output.contains("5 -> 4 [ label=\"1.00"));

    Ok(())
}

#[test]
fn test_not_in_all() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/not_in_all.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__"));
    assert!(output.contains("2 [ label=\"i2: a"));
    assert!(output.contains("3 [ label=\"i3: !b"));
    assert!(output.contains("4 [ label=\"i4: c"));
    assert!(output.contains("5 [ label=\"i5: !d"));
    assert!(output.contains("2 -> 0 [ label=\"0.50"));
    assert!(output.contains("3 -> 0 [ label=\"0.50"));
    assert!(output.contains("4 -> 0 [ label=\"0.50"));
    assert!(output.contains("5 -> 2 [ label=\"1.00"));
    assert!(output.contains("5 -> 3 [ label=\"1.00"));
    assert!(output.contains("5 -> 4 [ label=\"1.00"));

    Ok(())
}

#[test]
fn test_any_in_all() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/any_in_all.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__"));
    assert!(output.contains("2 [ label=\"i2: a"));
    assert!(output.contains("3 [ label=\"i3: !b"));
    assert!(output.contains("4 [ label=\"i4: c"));
    assert!(output.contains("5 [ label=\"i5: d"));
    assert!(output.contains("6 [ label=\"i6: e"));
    assert!(output.contains("2 -> 0 [ label=\"0.50"));
    assert!(output.contains("3 -> 0 [ label=\"0.50"));
    assert!(output.contains("4 -> 0 [ label=\"0.50"));
    assert!(output.contains("5 -> 2 [ label=\"1.00"));
    assert!(output.contains("5 -> 3 [ label=\"1.00"));
    assert!(output.contains("5 -> 4 [ label=\"1.00"));
    assert!(output.contains("6 -> 2 [ label=\"1.00"));
    assert!(output.contains("6 -> 3 [ label=\"1.00"));
    assert!(output.contains("6 -> 4 [ label=\"1.00"));

    Ok(())
}

#[test]
fn test_all_in_all() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/all_in_all.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__"));
    assert!(output.contains("2 [ label=\"i2: a"));
    assert!(output.contains("3 [ label=\"i3: !b"));
    assert!(output.contains("4 [ label=\"i4: c"));
    assert!(output.contains("5 [ label=\"i5: d"));
    assert!(output.contains("6 [ label=\"i6: e"));
    assert!(output.contains("2 -> 0 [ label=\"0.50"));
    assert!(output.contains("3 -> 0 [ label=\"0.50"));
    assert!(output.contains("4 -> 0 [ label=\"0.50"));
    assert!(output.contains("5 -> 2 [ label=\"0.50"));
    assert!(output.contains("5 -> 3 [ label=\"0.50"));
    assert!(output.contains("5 -> 4 [ label=\"0.50"));
    assert!(output.contains("6 -> 2 [ label=\"0.50"));
    assert!(output.contains("6 -> 3 [ label=\"0.50"));
    assert!(output.contains("6 -> 4 [ label=\"0.50"));

    Ok(())
}

// =============================================
// ==================== ANY ====================
// =============================================

#[test]
fn test_one_in_any() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/one_in_any.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__"));
    assert!(output.contains("2 [ label=\"i2: a"));
    assert!(output.contains("3 [ label=\"i3: !b"));
    assert!(output.contains("4 [ label=\"i4: c"));
    assert!(output.contains("5 [ label=\"i5: d"));
    assert!(output.contains("2 -> 0 [ label=\"0.50"));
    assert!(output.contains("3 -> 0 [ label=\"0.50"));
    assert!(output.contains("4 -> 0 [ label=\"1.00"));
    assert!(output.contains("5 -> 2 [ label=\"1.00"));
    assert!(output.contains("5 -> 3 [ label=\"1.00"));
    assert!(output.contains("5 -> 4 [ label=\"1.00"));

    Ok(())
}

#[test]
fn test_not_in_any() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/not_in_any.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__"));
    assert!(output.contains("2 [ label=\"i2: a"));
    assert!(output.contains("3 [ label=\"i3: !b"));
    assert!(output.contains("4 [ label=\"i4: c"));
    assert!(output.contains("5 [ label=\"i5: !d"));
    assert!(output.contains("2 -> 0 [ label=\"0.50"));
    assert!(output.contains("3 -> 0 [ label=\"0.50"));
    assert!(output.contains("4 -> 0 [ label=\"1.00"));
    assert!(output.contains("5 -> 2 [ label=\"1.00"));
    assert!(output.contains("5 -> 3 [ label=\"1.00"));
    assert!(output.contains("5 -> 4 [ label=\"1.00"));

    Ok(())
}

#[test]
fn test_any_in_any() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/any_in_any.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__"));
    assert!(output.contains("2 [ label=\"i2: a"));
    assert!(output.contains("3 [ label=\"i3: !b"));
    assert!(output.contains("4 [ label=\"i4: c"));
    assert!(output.contains("5 [ label=\"i5: d"));
    assert!(output.contains("6 [ label=\"i6: e"));
    assert!(output.contains("2 -> 0 [ label=\"0.50"));
    assert!(output.contains("3 -> 0 [ label=\"0.50"));
    assert!(output.contains("4 -> 0 [ label=\"1.00"));
    assert!(output.contains("5 -> 2 [ label=\"1.00"));
    assert!(output.contains("5 -> 3 [ label=\"1.00"));
    assert!(output.contains("5 -> 4 [ label=\"1.00"));
    assert!(output.contains("6 -> 2 [ label=\"1.00"));
    assert!(output.contains("6 -> 3 [ label=\"1.00"));
    assert!(output.contains("6 -> 4 [ label=\"1.00"));

    Ok(())
}

#[test]
fn test_all_in_any() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/all_in_any.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__"));
    assert!(output.contains("2 [ label=\"i2: a"));
    assert!(output.contains("3 [ label=\"i3: !b"));
    assert!(output.contains("4 [ label=\"i4: c"));
    assert!(output.contains("5 [ label=\"i5: d"));
    assert!(output.contains("6 [ label=\"i6: e"));
    assert!(output.contains("2 -> 0 [ label=\"0.50"));
    assert!(output.contains("3 -> 0 [ label=\"0.50"));
    assert!(output.contains("4 -> 0 [ label=\"1.00"));
    assert!(output.contains("5 -> 2 [ label=\"0.50"));
    assert!(output.contains("5 -> 3 [ label=\"0.50"));
    assert!(output.contains("5 -> 4 [ label=\"0.50"));
    assert!(output.contains("6 -> 2 [ label=\"0.50"));
    assert!(output.contains("6 -> 3 [ label=\"0.50"));
    assert!(output.contains("6 -> 4 [ label=\"0.50"));

    Ok(())
}
