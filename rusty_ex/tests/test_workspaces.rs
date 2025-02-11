mod utils;

use pretty_assertions::assert_eq;
use utils::run_with_cargo_bin;

#[test]
fn test_version_output() -> Result<(), String> {
    let (output, _) = run_with_cargo_bin("workspaces/simple_feature_no_weights", None, &["-V"])?;
    assert_eq!(output, "0.1.0-nightly-2024-12-01\n");
    Ok(())
}

#[test]
fn test_help_output() -> Result<(), String> {
    let (output, _) =
        run_with_cargo_bin("workspaces/simple_feature_no_weights", None, &["--help"])?;
    for options in &[
        "--print-crate",
        "--print-artifacts-tree",
        "--print-features-graph",
        "--print-terms-tree",
    ] {
        assert!(output.contains(options));
    }
    Ok(())
}

#[test]
fn test_simple_feature_no_weigths_features_graph() -> Result<(), String> {
    let (output, _) = run_with_cargo_bin(
        "workspaces/simple_feature_no_weights",
        None,
        &["--print-features-graph"],
    )?;

    // nodes
    assert!(output.contains("0 [ label=\"i0: __GLOBAL__"));
    assert!(output.contains("2 [ label=\"i2: aa"));
    assert!(output.contains("3 [ label=\"i3: !bb"));
    assert!(output.contains("4 [ label=\"i4: cc"));
    assert!(output.contains("5 [ label=\"i5: dd"));
    assert!(output.contains("6 [ label=\"i6: ee"));
    assert!(output.contains("7 [ label=\"i7: !ff"));

    // edges
    assert!(output.contains("6 -> 5 [ label=\"1.00"));
    assert!(output.contains("5 -> 4 [ label=\"1.00"));
    assert!(output.contains("7 -> 5 [ label=\"1.00"));
    assert!(output.contains("2 -> 0 [ label=\"1.00"));
    assert!(output.contains("3 -> 0 [ label=\"1.00"));
    assert!(output.contains("4 -> 0 [ label=\"1.00"));

    Ok(())
}

#[test]
fn test_simple_feature_weigths_features_graph() -> Result<(), String> {
    let (output, _) = run_with_cargo_bin(
        "workspaces/simple_feature_weights",
        None,
        &["--print-features-graph"],
    )?;

    // nodes
    assert!(output.contains("0 [ label=\"i0: __GLOBAL__"));
    assert!(output.contains("2 [ label=\"i2: aa"));
    assert!(output.contains("3 [ label=\"i3: bb"));
    assert!(output.contains("4 [ label=\"i4: cc"));
    assert!(output.contains("5 [ label=\"i5: ee"));
    assert!(output.contains("6 [ label=\"i6: !ff"));
    assert!(output.contains("7 [ label=\"i7: dd"));

    // edges
    assert!(output.contains("3 -> 2 [ label=\"1.00"));
    assert!(output.contains("4 -> 2 [ label=\"1.00"));
    assert!(output.contains("2 -> 0 [ label=\"1.00"));
    assert!(output.contains("5 -> 0 [ label=\"0.50"));
    assert!(output.contains("6 -> 0 [ label=\"0.50"));
    assert!(output.contains("7 -> 5 [ label=\"1.00"));
    assert!(output.contains("7 -> 6 [ label=\"1.00"));

    Ok(())
}
