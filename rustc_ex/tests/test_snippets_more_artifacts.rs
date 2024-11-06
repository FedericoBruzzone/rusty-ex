mod utils;

// use pretty_assertions::assert_eq;
use utils::run_with_cargo_bin_and_snippet;

const FOLDER: &str = "tests/snippets/more_artifacts";

// =============================================

#[test]
fn test_declaration() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/declaration.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: main\"]"));
    assert!(output.contains("2 [ label=\"i2: declaration\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_constant() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/constant.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: constant-1\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: constant-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_assignment() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/assignment.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: main\"]"));
    assert!(output.contains("2 [ label=\"i2: assignment\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_struct_declaration() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/struct_declaration.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: struct-declaration-1\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: struct-declaration-2\"]"));

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

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: struct-1-fields-1\"]"));
    assert!(output.contains("2 [ label=\"i2: struct-1-fields-2\"]"));
    assert!(output.contains("3 [ label=\"i3: main\"]"));
    assert!(output.contains("4 [ label=\"i4: struct-2-fields-1\"]"));
    assert!(output.contains("5 [ label=\"i5: struct-2-fields-2\"]"));

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

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: enum-declaration-1\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: enum-declaration-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_enum_declaration_fields() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/enum_declaration_fields.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: enum-1-fields-1\"]"));
    assert!(output.contains("2 [ label=\"i2: enum-1-fields-2\"]"));
    assert!(output.contains("3 [ label=\"i3: main\"]"));
    assert!(output.contains("4 [ label=\"i4: enum-2-fields-1\"]"));
    assert!(output.contains("5 [ label=\"i5: enum-2-fields-2\"]"));

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

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: function-definition-1\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: function-definition-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_function_call() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/function_call.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: main\"]"));
    assert!(output.contains("2 [ label=\"i2: function-call-1\"]"));
    assert!(output.contains("3 [ label=\"i3: function-call-2\"]"));
    assert!(output.contains("4 [ label=\"i4: function-call-3\"]"));
    assert!(output.contains("5 [ label=\"i5: function-call-4\"]"));

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

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: trait-1\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: trait-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_implementation() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/implementation.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: implementation-1\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: implementation-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_method_definition() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/method_definition.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: method-definition\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: static-method-definition\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_method_call() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/method_call.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: main\"]"));
    assert!(output.contains("2 [ label=\"i2: method-call-1\"]"));
    assert!(output.contains("3 [ label=\"i3: method-call-2\"]"));
    assert!(output.contains("4 [ label=\"i4: method-call-3\"]"));
    assert!(output.contains("5 [ label=\"i5: method-call-4\"]"));

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

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: main\"]"));
    assert!(output.contains("2 [ label=\"i2: block-1\"]"));
    assert!(output.contains("3 [ label=\"i3: block-2\"]"));
    assert!(output.contains("4 [ label=\"i4: block-3\"]"));

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

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: main\"]"));
    assert!(output.contains("2 [ label=\"i2: closure-1\"]"));
    assert!(output.contains("3 [ label=\"i3: closure-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 1 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_extern_crate() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/extern_crate.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: extern-crate\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_module() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/module.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: module\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_use() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/use.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: use\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_if() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/if.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: main\"]"));
    assert!(output.contains("2 [ label=\"i2: if-1\"]"));
    assert!(output.contains("3 [ label=\"i3: if-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_loop() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/loop.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: main\"]"));
    assert!(output.contains("2 [ label=\"i2: for\"]"));
    assert!(output.contains("3 [ label=\"i3: while\"]"));
    assert!(output.contains("4 [ label=\"i4: loop-1\"]"));
    assert!(output.contains("5 [ label=\"i5: loop-2\"]"));

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

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: main\"]"));
    assert!(output.contains("2 [ label=\"i2: match-1\"]"));
    assert!(output.contains("3 [ label=\"i3: match-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_match_branch() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/match_branch.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: main\"]"));
    assert!(output.contains("2 [ label=\"i2: match-branch-1\"]"));
    assert!(output.contains("3 [ label=\"i3: match-branch-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 1 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_return() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/return.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: return-1\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: return-2\"]"));
    assert!(output.contains("4 [ label=\"i4: return-3\"]"));

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

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: type-alias-1\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: type-alias-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_union() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/union.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: union-1\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: union-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_macro_call() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/macro_call.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: main\"]"));
    assert!(output.contains("2 [ label=\"i2: macro-call-1\"]"));
    assert!(output.contains("3 [ label=\"i3: macro-call-2\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 1 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_macro_definition() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/macro_definition.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("1 [ label=\"i1: macro-definition-1\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: macro-definition-2\"]"));
    assert!(output.contains("4 [ label=\"i4: macro-definition-3\"]"));

    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 2 [ label=\"1.00\"]"));

    Ok(())
}
