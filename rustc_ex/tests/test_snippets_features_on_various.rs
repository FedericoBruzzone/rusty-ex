mod utils;

// use pretty_assertions::assert_eq;
use utils::run_with_cargo_bin_and_snippet;

const FOLDER: &str = "tests/snippets/features_on_various";

// =============================================

#[test]
fn test_declaration() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/declaration.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: declaration\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_constant() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/constant.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: constant-1\"]"));
    assert!(output.contains("3 [ label=\"i3: main\"]"));
    assert!(output.contains("4 [ label=\"i4: constant-2\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 3 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_assignment() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/assignment.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: assignment\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_struct_declaration() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/struct_declaration.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: struct-declaration-1\"]"));
    assert!(output.contains("3 [ label=\"i3: main\"]"));
    assert!(output.contains("4 [ label=\"i4: struct-declaration-2\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 3 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_struct_declaration_fields() -> Result<(), String> {
    let snippet =
        &std::fs::read_to_string(format!("{FOLDER}/struct_declaration_fields.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: struct-1-fields-1\"]"));
    assert!(output.contains("3 [ label=\"i3: struct-1-fields-2\"]"));
    assert!(output.contains("4 [ label=\"i4: main\"]"));
    assert!(output.contains("5 [ label=\"i5: struct-2-fields-1\"]"));
    assert!(output.contains("6 [ label=\"i6: struct-2-fields-2\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("5 -> 4 [ label=\"1.00\"]"));
    assert!(output.contains("6 -> 4 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_enum_declaration() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/enum_declaration.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: enum-declaration-1\"]"));
    assert!(output.contains("3 [ label=\"i3: main\"]"));
    assert!(output.contains("4 [ label=\"i4: enum-declaration-2\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 3 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_enum_declaration_fields() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/enum_declaration_fields.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: enum-1-fields-1\"]"));
    assert!(output.contains("3 [ label=\"i3: enum-1-fields-2\"]"));
    assert!(output.contains("4 [ label=\"i4: main\"]"));
    assert!(output.contains("5 [ label=\"i5: enum-2-fields-1\"]"));
    assert!(output.contains("6 [ label=\"i6: enum-2-fields-2\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("5 -> 4 [ label=\"1.00\"]"));
    assert!(output.contains("6 -> 4 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_function_definition() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/function_definition.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: function-definition-1\"]"));
    assert!(output.contains("3 [ label=\"i3: main\"]"));
    assert!(output.contains("4 [ label=\"i4: function-definition-2\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 3 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_function_call() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/function_call.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: function-call-1\"]"));
    assert!(output.contains("4 [ label=\"i4: function-call-2\"]"));
    assert!(output.contains("5 [ label=\"i5: function-call-3\"]"));
    assert!(output.contains("6 [ label=\"i6: function-call-4\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 2 [ label=\"1.00\"]"));
    assert!(output.contains("5 -> 2 [ label=\"1.00\"]"));
    assert!(output.contains("6 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_trait() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/trait.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: trait-1\"]"));
    assert!(output.contains("3 [ label=\"i3: main\"]"));
    assert!(output.contains("4 [ label=\"i4: trait-2\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 3 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_implementation() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/implementation.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: implementation-1\"]"));
    assert!(output.contains("3 [ label=\"i3: main\"]"));
    assert!(output.contains("4 [ label=\"i4: implementation-2\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 3 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_method_definition() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/method_definition.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: method-definition\"]"));
    assert!(output.contains("3 [ label=\"i3: main\"]"));
    assert!(output.contains("4 [ label=\"i4: static-method-definition\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 3 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_method_call() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/method_call.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: method-call-1\"]"));
    assert!(output.contains("4 [ label=\"i4: method-call-2\"]"));
    assert!(output.contains("5 [ label=\"i5: method-call-3\"]"));
    assert!(output.contains("6 [ label=\"i6: method-call-4\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 2 [ label=\"1.00\"]"));
    assert!(output.contains("5 -> 2 [ label=\"1.00\"]"));
    assert!(output.contains("6 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_block() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/block.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: block-1\"]"));
    assert!(output.contains("4 [ label=\"i4: block-2\"]"));
    assert!(output.contains("5 [ label=\"i5: block-3\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 2 [ label=\"1.00\"]"));
    assert!(output.contains("5 -> 4 [ label=\"1.00\"]"));

    Ok(())
}
#[test]
fn test_closure() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/closure.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: closure-1\"]"));
    assert!(output.contains("4 [ label=\"i4: closure-2\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_extern_crate() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/extern_crate.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: extern-crate\"]"));
    assert!(output.contains("3 [ label=\"i3: main\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 0 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_module() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/module.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: module\"]"));
    assert!(output.contains("3 [ label=\"i3: main\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 0 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_use() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/use.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: use\"]"));
    assert!(output.contains("3 [ label=\"i3: main\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 0 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_if() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/if.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: if-1\"]"));
    assert!(output.contains("4 [ label=\"i4: if-2\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 3 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_loop() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/loop.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: for\"]"));
    assert!(output.contains("4 [ label=\"i4: while\"]"));
    assert!(output.contains("5 [ label=\"i5: loop-1\"]"));
    assert!(output.contains("6 [ label=\"i6: loop-2\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 3 [ label=\"1.00\"]"));
    assert!(output.contains("5 -> 4 [ label=\"1.00\"]"));
    assert!(output.contains("6 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_match() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/match.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: match-1\"]"));
    assert!(output.contains("4 [ label=\"i4: match-2\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 3 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_match_branch() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/match_branch.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: match-branch-1\"]"));
    assert!(output.contains("4 [ label=\"i4: match-branch-2\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_return() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/return.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: return-1\"]"));
    assert!(output.contains("3 [ label=\"i3: main\"]"));
    assert!(output.contains("4 [ label=\"i4: return-2\"]"));
    assert!(output.contains("5 [ label=\"i5: return-3\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 3 [ label=\"1.00\"]"));
    assert!(output.contains("5 -> 3 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_type_alias() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/type_alias.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: type-alias-1\"]"));
    assert!(output.contains("3 [ label=\"i3: main\"]"));
    assert!(output.contains("4 [ label=\"i4: type-alias-2\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 3 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_union() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/union.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: union-1\"]"));
    assert!(output.contains("3 [ label=\"i3: main\"]"));
    assert!(output.contains("4 [ label=\"i4: union-2\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 3 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_macro_call() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/macro_call.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: main\"]"));
    assert!(output.contains("3 [ label=\"i3: macro-call-1\"]"));
    assert!(output.contains("4 [ label=\"i4: macro-call-2\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 2 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 2 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
fn test_macro_definition() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/macro_definition.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-graph"])?;

    assert!(output.contains("0 [ label=\"i0: __GLOBAL__\"]"));
    assert!(output.contains("2 [ label=\"i2: macro-definition-1\"]"));
    assert!(output.contains("3 [ label=\"i3: main\"]"));
    assert!(output.contains("4 [ label=\"i4: macro-definition-2\"]"));
    assert!(output.contains("5 [ label=\"i5: macro-definition-3\"]"));

    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 3 [ label=\"1.00\"]"));
    assert!(output.contains("5 -> 3 [ label=\"1.00\"]"));

    Ok(())
}
