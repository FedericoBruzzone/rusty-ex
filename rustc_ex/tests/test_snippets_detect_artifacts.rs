mod utils;

use utils::run_with_cargo_bin_and_snippet;
use utils::{count_line, same_line};

const FOLDER: &str = "tests/snippets/detect_artifacts";

// =============================================

#[test]
#[ignore]
fn test_macro() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/macro.rs")).unwrap();
    let (_output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    // FIXME: le macro sono già state espanse

    Ok(())
}

#[test]
fn test_module() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/module.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Mod", "example1", "w0.00"]));
    assert!(same_line(&output, vec!["Mod", "example2", "w0.00"]));
    assert!(same_line(&output, vec!["Mod", "example3", "w2.00"]));
    assert!(same_line(&output, vec!["Mod", "example4", "w2.00"]));

    Ok(())
}

#[test]
fn test_extern_crate() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/extern_crate.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["ExternCrate", "example1", "w0.00"]));
    assert!(same_line(&output, vec!["ExternCrate", "example2", "w0.00"]));

    Ok(())
}

#[test]
fn test_use() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/use.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    // TODO: serve sapere anche le path degli import?
    assert_eq!(count_line(&output, vec!["Use", "w0.00"]), 10 + 1); // 10 from the snippet + 1 always present

    Ok(())
}

#[test]
fn test_function() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/function.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Fn", "example1", "w0.00"]));
    assert!(same_line(&output, vec!["Fn", "example2", "w0.00"]));
    assert!(same_line(&output, vec!["Fn", "example3", "w0.00"]));
    assert!(same_line(&output, vec!["Fn", "example4", "w0.00"]));
    assert!(same_line(&output, vec!["Fn", "example5", "w0.00"]));
    assert!(same_line(&output, vec!["Fn", "example6", "w0.00"]));
    assert!(same_line(&output, vec!["Fn", "example7", "w0.00"]));
    assert!(same_line(&output, vec!["Fn", "example7", "w0.00"]));
    assert!(same_line(&output, vec!["Fn", "example8", "w0.00"]));
    assert!(same_line(&output, vec!["Fn", "example9", "w0.00"]));
    assert!(same_line(&output, vec!["Fn", "example10", "w0.00"]));
    assert!(same_line(&output, vec!["Fn", "example11", "w0.00"]));
    assert!(same_line(&output, vec!["Fn", "example12", "w0.00"]));
    assert!(same_line(&output, vec!["Fn", "example13", "w1.00"]));

    Ok(())
}

#[test]
fn test_type() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/type.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["TyAlias", "Example1", "w1.00"]));
    assert!(same_line(&output, vec!["TyAlias", "Example2", "w1.00"]));
    assert!(same_line(&output, vec!["TyAlias", "Example3", "w1.00"]));
    assert!(same_line(&output, vec!["TyAlias", "Example4", "w1.00"]));

    Ok(())
}

#[test]
fn test_struct() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/struct.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Struct", "Example1", "w0.00"]));
    assert!(same_line(&output, vec!["Struct", "Example2", "w2.00"]));
    assert!(same_line(&output, vec!["Struct", "Example3", "w1.00"]));
    assert!(same_line(&output, vec!["Struct", "Example4", "w2.00"]));

    Ok(())
}

#[test]
fn test_enum() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/enum.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Enum", "Example1", "w0.00"]));
    assert!(same_line(&output, vec!["Enum", "Example2", "w2.00"]));
    assert!(same_line(&output, vec!["Enum", "Example3", "w6.00"]));
    assert!(same_line(&output, vec!["Enum", "Example4", "w4.00"]));
    assert!(same_line(&output, vec!["Enum", "Example5", "w8.00"]));
    assert!(same_line(&output, vec!["Enum", "Example6", "w4.00"]));

    Ok(())
}

#[test]
fn test_union() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/union.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Union", "Example1", "w2.00"]));
    assert!(same_line(&output, vec!["Union", "Example2", "w0.00"]));
    assert!(same_line(&output, vec!["Union", "Example3", "w2.00"]));

    Ok(())
}

#[test]
fn test_const() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/const.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Const", "EXAMPLE1", "w3.00"]));
    assert!(same_line(&output, vec!["Const", "EXAMPLE2", "w2.00"]));
    assert!(same_line(&output, vec!["Const", "EXAMPLE3", "w2.00"]));
    assert!(same_line(&output, vec!["Const", "EXAMPLE4", "w1.00"]));

    Ok(())
}

#[test]
fn test_static() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/static.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Static", "EXAMPLE1", "w2.00"]));
    assert!(same_line(&output, vec!["Static", "EXAMPLE2", "w2.00"]));
    assert!(same_line(&output, vec!["Static", "EXAMPLE3", "w2.00"]));
    assert!(same_line(&output, vec!["Static", "EXAMPLE4", "w1.00"]));

    Ok(())
}

#[test]
fn test_trait() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/trait.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Trait", "Example1", "w0.00"]));
    assert!(same_line(&output, vec!["Trait", "Example2", "w3.00"]));
    assert!(same_line(&output, vec!["Trait", "Example3", "w0.00"]));
    assert!(same_line(&output, vec!["Trait", "Example4", "w0.00"]));
    assert!(same_line(&output, vec!["Trait", "Example5", "w0.00"]));
    assert!(same_line(&output, vec!["Trait", "Example6", "w0.00"]));
    assert!(same_line(&output, vec!["Trait", "Example7", "w0.00"]));

    Ok(())
}

#[test]
fn test_impl() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/impl.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert_eq!(count_line(&output, vec!["Impl", "w0.00"]), 3);
    assert_eq!(count_line(&output, vec!["Impl", "w2.00"]), 1);
    assert_eq!(count_line(&output, vec!["Impl", "w4.00"]), 1);

    Ok(())
}
