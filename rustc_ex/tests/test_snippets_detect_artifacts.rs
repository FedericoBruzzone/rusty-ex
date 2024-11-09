mod utils;

use utils::run_with_cargo_bin_and_snippet;
use utils::{count_line, same_line};

const FOLDER: &str = "tests/snippets/detect_artifacts";

// =============================================

#[test]
fn test_macro() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/macro.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert_eq!(count_line(&output, vec!["MacCall"]), 3);

    Ok(())
}

#[test]
fn test_module() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/module.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Mod", "example1"]));
    assert!(same_line(&output, vec!["Mod", "example2"]));
    assert!(same_line(&output, vec!["Mod", "example3"]));
    assert!(same_line(&output, vec!["Mod", "example4"]));

    Ok(())
}

#[test]
fn test_extern_crate() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/extern_crate.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["ExternCrate", "example1"]));
    assert!(same_line(&output, vec!["ExternCrate", "example2"]));

    Ok(())
}

#[test]
fn test_use() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/use.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert_eq!(count_line(&output, vec!["Use"]), 10 + 1); // 10 from the snippet + 1 always present

    Ok(())
}

#[test]
fn test_function() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/function.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Fn", "example1"]));
    assert!(same_line(&output, vec!["Fn", "example2"]));
    assert!(same_line(&output, vec!["Fn", "example3"]));
    assert!(same_line(&output, vec!["Fn", "example4"]));
    assert!(same_line(&output, vec!["Fn", "example5"]));
    assert!(same_line(&output, vec!["Fn", "example6"]));
    assert!(same_line(&output, vec!["Fn", "example7"]));
    assert!(same_line(&output, vec!["Fn", "example7"]));
    assert!(same_line(&output, vec!["Fn", "example8"]));
    assert!(same_line(&output, vec!["Fn", "example9"]));
    assert!(same_line(&output, vec!["Fn", "example10"]));
    assert!(same_line(&output, vec!["Fn", "example11"]));
    assert!(same_line(&output, vec!["Fn", "example12"]));
    assert!(same_line(&output, vec!["Fn", "example13"]));

    Ok(())
}

#[test]
fn test_type() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/type.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["TyAlias", "Example1"]));
    assert!(same_line(&output, vec!["TyAlias", "Example2"]));
    assert!(same_line(&output, vec!["TyAlias", "Example3"]));
    assert!(same_line(&output, vec!["TyAlias", "Example4"]));

    Ok(())
}

#[test]
fn test_struct() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/struct.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Struct", "Example1"]));
    assert!(same_line(&output, vec!["Struct", "Example2"]));
    assert!(same_line(&output, vec!["Struct", "Example3"]));
    assert!(same_line(&output, vec!["Struct", "Example4"]));

    Ok(())
}

#[test]
fn test_enum() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/enum.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Enum", "Example1"]));
    assert!(same_line(&output, vec!["Enum", "Example2"]));
    assert!(same_line(&output, vec!["Enum", "Example3"]));
    assert!(same_line(&output, vec!["Enum", "Example4"]));
    assert!(same_line(&output, vec!["Enum", "Example5"]));
    assert!(same_line(&output, vec!["Enum", "Example6"]));

    Ok(())
}

#[test]
fn test_union() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/union.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Union", "Example1"]));
    assert!(same_line(&output, vec!["Union", "Example2"]));
    assert!(same_line(&output, vec!["Union", "Example3"]));

    Ok(())
}

#[test]
fn test_const() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/const.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Const", "EXAMPLE1"]));
    assert!(same_line(&output, vec!["Const", "EXAMPLE2"]));
    assert!(same_line(&output, vec!["Const", "EXAMPLE3"]));
    assert!(same_line(&output, vec!["Const", "EXAMPLE4"]));

    Ok(())
}

#[test]
fn test_static() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/static.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Static", "EXAMPLE1"]));
    assert!(same_line(&output, vec!["Static", "EXAMPLE2"]));
    assert!(same_line(&output, vec!["Static", "EXAMPLE3"]));
    assert!(same_line(&output, vec!["Static", "EXAMPLE4"]));

    Ok(())
}

#[test]
fn test_trait() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/trait.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Trait", "Example1"]));
    assert!(same_line(&output, vec!["Trait", "Example2"]));
    assert!(same_line(&output, vec!["Trait", "Example3"]));
    assert!(same_line(&output, vec!["Trait", "Example4"]));
    assert!(same_line(&output, vec!["Trait", "Example5"]));
    assert!(same_line(&output, vec!["Trait", "Example6"]));
    assert!(same_line(&output, vec!["Trait", "Example7"]));

    Ok(())
}

#[test]
fn test_impl() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/impl.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert_eq!(count_line(&output, vec!["Impl"]), 5);

    Ok(())
}

#[test]
fn test_assoc_item() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/assoc_item.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert!(same_line(&output, vec!["Fn", "example1"]));
    assert!(same_line(&output, vec!["Fn", "example2"]));
    assert!(same_line(&output, vec!["Fn", "example3"]));
    assert!(same_line(&output, vec!["Fn", "example4"]));
    assert!(same_line(&output, vec!["Fn", "example5"]));
    assert!(same_line(&output, vec!["Fn", "example6"]));
    assert!(same_line(&output, vec!["Type", "Example1"]));
    assert!(same_line(&output, vec!["Type", "Example2"]));
    assert!(same_line(&output, vec!["Type", "Example3"]));
    assert!(same_line(&output, vec!["Type", "Example4"]));
    assert!(same_line(&output, vec!["Const", "EXAMPLE1"]));
    assert!(same_line(&output, vec!["Const", "EXAMPLE2"]));

    Ok(())
}

#[test]
fn test_statement() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/statement.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert_eq!(count_line(&output, vec!["Let"]), 3);

    Ok(())
}

#[test]
fn test_literal() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/literal.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert_eq!(count_line(&output, vec!["Lit"]), 61);

    Ok(())
}

#[test]
fn test_path() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/path.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert_eq!(count_line(&output, vec!["Path"]), 6);

    Ok(())
}

#[test]
fn test_block() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/block.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert_eq!(count_line(&output, vec!["Block("]), 13);

    Ok(())
}

#[test]
fn test_opeartor() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/operator.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert_eq!(count_line(&output, vec!["Semi"]), 42);

    Ok(())
}

#[test]
fn test_closure() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/closure.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert_eq!(count_line(&output, vec!["Closure"]), 3);

    Ok(())
}

#[test]
fn test_loop() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/loop.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert_eq!(count_line(&output, vec!["(Loop"]), 3);
    assert_eq!(count_line(&output, vec!["While"]), 2);
    assert_eq!(count_line(&output, vec!["ForLoop"]), 1);
    assert_eq!(count_line(&output, vec!["Break"]), 1);
    assert_eq!(count_line(&output, vec!["Continue"]), 1);

    Ok(())
}

#[test]
fn test_range() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/range.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert_eq!(count_line(&output, vec!["Range"]), 6);

    Ok(())
}

#[test]
fn test_if() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/if.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert_eq!(count_line(&output, vec!["If"]), 9);

    Ok(())
}

#[test]
fn test_match() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/match.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert_eq!(count_line(&output, vec!["Match"]), 3);
    assert_eq!(count_line(&output, vec!["Arm"]), 7);

    Ok(())
}

#[test]
fn test_return() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/return.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert_eq!(count_line(&output, vec!["Ret"]), 2);

    Ok(())
}

#[test]
fn test_await() -> Result<(), String> {
    let snippet = &std::fs::read_to_string(format!("{FOLDER}/await.rs")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-ast-graph"])?;

    assert_eq!(count_line(&output, vec!["Await"]), 1);

    Ok(())
}
