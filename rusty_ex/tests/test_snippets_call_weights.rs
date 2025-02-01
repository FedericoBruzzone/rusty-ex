mod utils;

use utils::run_with_cargo_bin_and_snippet;
use utils::same_line;

const FOLDER: &str = "tests/snippets/call_weights";

// =============================================

#[test]
fn test_call_to_after() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/call_to_after.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Fn", "'a'", "w1.00"]));
    assert!(same_line(&output, vec!["Call(Call)->a", "w2.00"]));

    Ok(())
}

#[test]
fn test_call_to_before() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/call_to_before.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Fn", "'a'", "w1.00"]));
    assert!(same_line(&output, vec!["Call(Call)->a", "w2.00"]));

    Ok(())
}

#[test]
fn test_call_to_inner_after() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/call_to_inner_after.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Fn", "'a'", "w1.00"]));
    assert!(same_line(&output, vec!["Call(Call)->a", "w2.00"]));

    Ok(())
}

#[test]
fn test_call_to_inner_before() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/call_to_inner_before.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Fn", "'a'", "w1.00"]));
    assert!(same_line(&output, vec!["Call(Call)->a", "w2.00"]));

    Ok(())
}

#[test]
fn test_double_call1() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/double_call1.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Fn", "'b'", "w1.00"]));
    assert!(same_line(&output, vec!["Call(Call)->b", "w2.00"]));
    assert!(same_line(&output, vec!["Fn", "'a'", "w2.00"]));
    assert!(same_line(&output, vec!["Call(Call)->a", "w3.00"]));

    Ok(())
}

#[test]
fn test_double_call2() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/double_call2.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Fn", "'b'", "w1.00"]));
    assert!(same_line(&output, vec!["Call(Call)->b", "w2.00"]));
    assert!(same_line(&output, vec!["Fn", "'a'", "w2.00"]));
    assert!(same_line(&output, vec!["Call(Call)->a", "w3.00"]));

    Ok(())
}

#[test]
fn test_multiple_calls1() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/multiple_calls1.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Fn", "'f'", "w1.00"]));
    assert!(same_line(&output, vec!["Fn", "'e'", "w2.00"]));
    assert!(same_line(&output, vec!["Fn", "'d'", "w3.00"]));
    assert!(same_line(&output, vec!["Fn", "'c'", "w4.00"]));
    assert!(same_line(&output, vec!["Fn", "'b'", "w5.00"]));
    assert!(same_line(&output, vec!["Fn", "'a'", "w6.00"]));

    assert!(same_line(&output, vec!["Call(Call)->f", "w2.00"]));
    assert!(same_line(&output, vec!["Call(Call)->e", "w3.00"]));
    assert!(same_line(&output, vec!["Call(Call)->d", "w4.00"]));
    assert!(same_line(&output, vec!["Call(Call)->c", "w5.00"]));
    assert!(same_line(&output, vec!["Call(Call)->b", "w6.00"]));
    assert!(same_line(&output, vec!["Call(Call)->a", "w7.00"]));

    Ok(())
}

#[test]
fn test_multiple_calls2() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/multiple_calls2.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Fn", "'f'", "w1.00"]));
    assert!(same_line(&output, vec!["Fn", "'e'", "w2.00"]));
    assert!(same_line(&output, vec!["Fn", "'d'", "w3.00"]));
    assert!(same_line(&output, vec!["Fn", "'c'", "w4.00"]));
    assert!(same_line(&output, vec!["Fn", "'b'", "w5.00"]));
    assert!(same_line(&output, vec!["Fn", "'a'", "w6.00"]));

    assert!(same_line(&output, vec!["Call(Call)->f", "w2.00"]));
    assert!(same_line(&output, vec!["Call(Call)->e", "w3.00"]));
    assert!(same_line(&output, vec!["Call(Call)->d", "w4.00"]));
    assert!(same_line(&output, vec!["Call(Call)->c", "w5.00"]));
    assert!(same_line(&output, vec!["Call(Call)->b", "w6.00"]));
    assert!(same_line(&output, vec!["Call(Call)->a", "w7.00"]));

    Ok(())
}

#[test]
fn test_recursive() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/recursive.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Fn", "'main'", "w8.00"])); // resolved with recovery mode

    assert!(same_line(&output, vec!["Call(Call)->main", "w8.00"]));

    Ok(())
}

#[test]
fn test_mutual_recursive() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/mutual_recursive.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Fn", "'main'", "w9.00"]));
    assert!(same_line(&output, vec!["Fn", "'a'", "9.00"]));

    assert!(same_line(&output, vec!["Call(Call)->a", "w9.00"]));
    assert!(same_line(&output, vec!["Call(Call)->main", "w9.00"]));

    Ok(())
}
