mod utils;

// use pretty_assertions::assert_eq;
use utils::run_with_cargo_bin_and_snippet;

const FOLDER: &str = "tests/snippets/more_artifacts";

// =============================================

#[test]
fn test_declaration() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/declaration.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"main\"]"));
    assert!(output.contains("2 [ label=\"declaration\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_constant() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/constant.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"constant-1\"]"));
    assert!(output.contains("2 [ label=\"main\"]"));
    assert!(output.contains("3 [ label=\"constant-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_assignment() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/assignment.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"main\"]"));
    assert!(output.contains("2 [ label=\"assignment\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_struct_declaration() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/struct_declaration.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"struct-declaration-1\"]"));
    assert!(output.contains("2 [ label=\"main\"]"));
    assert!(output.contains("3 [ label=\"struct-declaration-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_struct_declaration_fields() -> Result<(), String> {
    let snippet =
        &std::fs::read_to_string(format!("{FOLDER}/struct_declaration_fields.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"struct-1-fields-1\"]"));
    assert!(output.contains("2 [ label=\"struct-1-fields-2\"]"));
    assert!(output.contains("3 [ label=\"main\"]"));
    assert!(output.contains("4 [ label=\"struct-2-fields-1\"]"));
    assert!(output.contains("5 [ label=\"struct-2-fields-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 3 [ label=\"1.00\"]"));
    assert!(output.contains("5 -> 3 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_enum_declaration() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/enum_declaration.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"enum-declaration-1\"]"));
    assert!(output.contains("2 [ label=\"main\"]"));
    assert!(output.contains("3 [ label=\"enum-declaration-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_enum_declaration_fields() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/enum_declaration_fields.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"enum-1-fields-1\"]"));
    assert!(output.contains("2 [ label=\"enum-1-fields-2\"]"));
    assert!(output.contains("3 [ label=\"main\"]"));
    assert!(output.contains("4 [ label=\"enum-2-fields-1\"]"));
    assert!(output.contains("5 [ label=\"enum-2-fields-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 3 [ label=\"1.00\"]"));
    assert!(output.contains("5 -> 3 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_function_definition() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/function_definition.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"function-definition-1\"]"));
    assert!(output.contains("2 [ label=\"main\"]"));
    assert!(output.contains("3 [ label=\"function-definition-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_function_call() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/function_call.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"main\"]"));
    assert!(output.contains("2 [ label=\"function-call-1\"]"));
    assert!(output.contains("3 [ label=\"function-call-2\"]"));
    assert!(output.contains("4 [ label=\"function-call-3\"]"));
    assert!(output.contains("5 [ label=\"function-call-4\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("5 -> 1 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_trait() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/trait.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"trait-1\"]"));
    assert!(output.contains("2 [ label=\"main\"]"));
    assert!(output.contains("3 [ label=\"trait-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_implementation() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/implementation.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"implementation-1\"]"));
    assert!(output.contains("2 [ label=\"main\"]"));
    assert!(output.contains("3 [ label=\"implementation-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_method_definition() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/method_definition.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"method-definition\"]"));
    assert!(output.contains("2 [ label=\"main\"]"));
    assert!(output.contains("3 [ label=\"static-method-definition\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_method_call() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/method_call.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"main\"]"));
    assert!(output.contains("2 [ label=\"method-call-1\"]"));
    assert!(output.contains("3 [ label=\"method-call-2\"]"));
    assert!(output.contains("4 [ label=\"method-call-3\"]"));
    assert!(output.contains("5 [ label=\"method-call-4\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("5 -> 1 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_block() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/block.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"main\"]"));
    assert!(output.contains("2 [ label=\"block-1\"]"));
    assert!(output.contains("3 [ label=\"block-2\"]"));
    assert!(output.contains("4 [ label=\"block-3\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 3 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_closure() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/closure.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"main\"]"));
    assert!(output.contains("2 [ label=\"closure-1\"]"));
    assert!(output.contains("3 [ label=\"closure-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 1 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_extern_crate() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/extern_crate.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"extern-crate\"]"));
    assert!(output.contains("2 [ label=\"main\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_module() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/module.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"module\"]"));
    assert!(output.contains("2 [ label=\"main\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_use() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/use.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"use\"]"));
    assert!(output.contains("2 [ label=\"main\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_if() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/if.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"main\"]"));
    assert!(output.contains("2 [ label=\"if-1\"]"));
    assert!(output.contains("3 [ label=\"if-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_loop() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/loop.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"main\"]"));
    assert!(output.contains("2 [ label=\"for\"]"));
    assert!(output.contains("3 [ label=\"while\"]"));
    assert!(output.contains("4 [ label=\"loop-1\"]"));
    assert!(output.contains("5 [ label=\"loop-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 3 [ label=\"1.00\"]"));
    assert!(output.contains("5 -> 1 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_match() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/match.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"main\"]"));
    assert!(output.contains("2 [ label=\"match-1\"]"));
    assert!(output.contains("3 [ label=\"match-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_match_branch() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/match_branch.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"main\"]"));
    assert!(output.contains("2 [ label=\"match-branch-1\"]"));
    assert!(output.contains("3 [ label=\"match-branch-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 1 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_return() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/return.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"return-1\"]"));
    assert!(output.contains("2 [ label=\"main\"]"));
    assert!(output.contains("3 [ label=\"return-2\"]"));
    assert!(output.contains("4 [ label=\"return-3\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_type_alias() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/type_alias.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"type-alias-1\"]"));
    assert!(output.contains("2 [ label=\"main\"]"));
    assert!(output.contains("3 [ label=\"type-alias-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_union() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/union.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"union-1\"]"));
    assert!(output.contains("2 [ label=\"main\"]"));
    assert!(output.contains("3 [ label=\"union-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_macro_call() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/macro_call.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"main\"]"));
    assert!(output.contains("2 [ label=\"macro-call-1\"]"));
    assert!(output.contains("3 [ label=\"macro-call-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 1 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_macro_definition() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/macro_definition.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"__GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"macro-definition-1\"]"));
    assert!(output.contains("2 [ label=\"main\"]"));
    assert!(output.contains("3 [ label=\"macro-definition-2\"]"));
    assert!(output.contains("4 [ label=\"macro-definition-3\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 2 [ label=\"1.00\"]"));

    Ok(())
}
