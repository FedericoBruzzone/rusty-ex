mod utils;

use pretty_assertions::assert_eq;
use utils::run_with_cargo_bin;

#[test]
fn test_version_output() -> Result<(), String> {
    let (output, _) = run_with_cargo_bin("workspaces/simple_feature_no_weights", None, &["-V"])?;
    assert_eq!(output, "0.1.0-nightly-2024-10-18\n");
    Ok(())
}

#[test]
fn test_help_output() -> Result<(), String> {
    let (output, _) =
        run_with_cargo_bin("workspaces/simple_feature_no_weights", None, &["--help"])?;
    for options in &[
        "--print-crate",
        "--print-artifacts-graph",
        "--print-features-graph",
        "--print-ast-graph",
    ] {
        assert!(output.contains(options));
    }
    Ok(())
}

#[test]
#[ignore] // TODO: test grafo degli artefatti dopo averlo definito bene
fn test_simple_feature_no_weigths_artifacts_dot() -> Result<(), String> {
    let (output, _) = run_with_cargo_bin(
        "workspaces/simple_feature_no_weights",
        None,
        &["--print-artifacts-dot"],
    )?;

    assert!(output.contains(
        r#"digraph {
    0 [ label="__GLOBAL__ #[__GLOBAL__]"]
    1 [ label="one #[aa]"]
    2 [ label="two #[not(bb)]"]
    3 [ label="three #[cc]"]
    4 [ label="four #[dd]"]
    5 [ label="five #[ee]"]
    6 [ label="six #[not(ff)]"]
    1 -> 0 [ label="0"]
    2 -> 0 [ label="0"]
    5 -> 4 [ label="0"]
    6 -> 4 [ label="0"]
    4 -> 3 [ label="0"]
    3 -> 0 [ label="0"]
}"#
    ));

    Ok(())
}

#[test]
fn test_simple_feature_no_weigths_features_dot() -> Result<(), String> {
    let (output, _) = run_with_cargo_bin(
        "workspaces/simple_feature_no_weights",
        None,
        &["--print-features-dot"],
    )?;

    assert!(output.contains(
        r#"digraph {
    0 [ label="__GLOBAL__"]
    1 [ label="aa"]
    2 [ label="!bb"]
    3 [ label="cc"]
    4 [ label="dd"]
    5 [ label="ee"]
    6 [ label="!ff"]"#
    ));

    // edges order is not deterministic
    assert!(output.contains("5 -> 4 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 3 [ label=\"1.00\"]"));
    assert!(output.contains("6 -> 4 [ label=\"1.00\"]"));
    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("2 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 0 [ label=\"1.00\"]"));

    Ok(())
}

#[test]
#[ignore] // TODO: test grafo degli artefatti dopo averlo definito bene
fn test_simple_feature_weigths_artifacts_dot() -> Result<(), String> {
    let (output, _) = run_with_cargo_bin(
        "workspaces/simple_feature_weights",
        None,
        &["--print-artifacts-dot"],
    )?;

    assert!(output.contains(
        r#"digraph {
    0 [ label="__GLOBAL__ #[__GLOBAL__]"]
    1 [ label="one #[aa]"]
    2 [ label="two #[any(bb, cc)]"]
    3 [ label="three #[all(ee, not(ff))]"]
    4 [ label="four #[dd]"]
    2 -> 1 [ label="0"]
    1 -> 0 [ label="0"]
    4 -> 3 [ label="0"]
    3 -> 0 [ label="0"]
}"#
    ));

    Ok(())
}

#[test]
fn test_simple_feature_weigths_features_dot() -> Result<(), String> {
    let (output, _) = run_with_cargo_bin(
        "workspaces/simple_feature_weights",
        None,
        &["--print-features-dot"],
    )?;

    assert!(output.contains(
        r#"digraph {
    0 [ label="__GLOBAL__"]
    1 [ label="aa"]
    2 [ label="bb"]
    3 [ label="cc"]
    4 [ label="ee"]
    5 [ label="!ff"]
    6 [ label="dd"]"#
    ));

    // edges order is not deterministic
    assert!(output.contains("2 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("3 -> 1 [ label=\"1.00\"]"));
    assert!(output.contains("1 -> 0 [ label=\"1.00\"]"));
    assert!(output.contains("4 -> 0 [ label=\"0.50\"]"));
    assert!(output.contains("5 -> 0 [ label=\"0.50\"]"));
    assert!(output.contains("6 -> 4 [ label=\"1.00\"]"));
    assert!(output.contains("6 -> 5 [ label=\"1.00\"]"));

    Ok(())
}
