use std::fs;
use std::process::Command;
use std::sync::Once;
use std::{env, path::Path};

const PLUGIN_NAME: &str = "rustc-ex";
const TEST_MODE_FEATURE: &str = "test-mode";
static INSTALL_PLUGIN: Once = Once::new();

#[cfg(test)]
/// Run the plugin with the `cargo` command
///
/// This function will install the plugin (cargo-rustc-ex binary) in a temporary directory and run it with the `cargo` command.
/// The plugin will be installed only once.
///
/// # Arguments
/// * `cargo_project_name` - The name of the cargo project in the `tests` directory. E.g. `workspaces/simple_feature_no_weights`
/// * `expected_outout_name` - The name of the file containing the expected output in the cargo project directory. E.g. `expected_output.stdout`
/// * `plugin_args` - The arguments to pass to the plugin
fn run_with_cargo_bin(
    cargo_project_name: &str,
    expected_outout_name: Option<&str>,
    plugin_args: &[&str],
) -> Result<(String, Option<String>), String> {
    // Install the plugin
    let root_dir = env::temp_dir().join("rustc-ex");
    let current_dir = Path::new(".").canonicalize().unwrap();
    INSTALL_PLUGIN.call_once(|| {
        let mut cargo_cmd = Command::new("cargo");
        cargo_cmd.args(["install", "--path", ".", "--debug", "--locked", "--root"]);
        cargo_cmd.arg(&root_dir);
        cargo_cmd.current_dir(&current_dir);
        // See the `args` function on `impl RustcPlugin for RustcEx` for the explanation of why we need to pass the `--features test-mode` argument.
        cargo_cmd.args(["--features", TEST_MODE_FEATURE]);
        let status = cargo_cmd.status().unwrap();
        if !status.success() {
            panic!("Failed to install the plugin");
        }
    });

    // Prepare the cargo command
    let path = format!(
        "{}:{}",
        root_dir.join("bin").display(),
        env::var("PATH").unwrap_or_default()
    );
    let workspace_path = current_dir.join("tests").join(cargo_project_name);
    let mut cargo_cmd = Command::new("cargo");
    cargo_cmd.arg(PLUGIN_NAME);
    for arg in plugin_args {
        cargo_cmd.arg(arg);
    }
    cargo_cmd.env("PATH", path);
    cargo_cmd.current_dir(&workspace_path);

    // Clean the target directory of the workspace
    let _ = fs::remove_dir_all(workspace_path.join("target"));

    // Run the plugin
    let output = cargo_cmd.output().unwrap();
    // assert!(output.status.success());  This cannot be true because the plugin is change all `#[cfg(` to `#[my_cfg(` in order to process all the features

    if let Some(expected_outout_name) = expected_outout_name {
        let expected_output_path = workspace_path.join(expected_outout_name);
        let expected_output = fs::read_to_string(expected_output_path).unwrap();
        Ok((
            String::from_utf8(output.stdout).unwrap(),
            Some(expected_output),
        ))
    } else {
        Ok((String::from_utf8(output.stdout).unwrap(), None))
    }
}

#[cfg(test)]
mod test_using_workspace_folders {

    use crate::run_with_cargo_bin;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_version_output() -> Result<(), String> {
        let (output, _) =
            run_with_cargo_bin("workspaces/simple_feature_no_weights", None, &["-V"])?;
        assert_eq!(output, "0.1.0-nightly-2024-10-18\n");
        Ok(())
    }

    #[test]
    fn test_help_output() -> Result<(), String> {
        let (output, _) =
            run_with_cargo_bin("workspaces/simple_feature_no_weights", None, &["--help"])?;
        for options in &[
            "--print-crate",
            "--print-artifacts-dot",
            "--print-features-dot",
        ] {
            assert!(output.contains(options));
        }
        Ok(())
    }

    #[test]
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
        assert!(output.contains("6 -> 4 [ label=\"2.00\"]"));
        assert!(output.contains("6 -> 5 [ label=\"2.00\"]"));

        Ok(())
    }
}

#[cfg(test)]
mod test_using_snippets {

    // use pretty_assertions::assert_eq;
    use crate::run_with_cargo_bin;
    use std::fs;
    use std::path::Path;

    fn create_cargo_project_with_snippet(snippet: &str) -> Result<(), String> {
        let current_dir = Path::new(".").canonicalize().unwrap();
        let workspace_path = current_dir.join("tests").join("workspaces").join("temp");
        fs::create_dir_all(&workspace_path).unwrap();
        let lib_rs_path = workspace_path.join("src").join("lib.rs");
        fs::create_dir_all(lib_rs_path.parent().unwrap()).unwrap();
        fs::write(lib_rs_path, snippet).unwrap();
        let manifest_path = workspace_path.join("Cargo.toml");
        fs::write(
            manifest_path,
            r#"
[package]
name = "temp"
version = "0.1.0"
edition = "2018"

[dependencies]
"#,
        )
        .unwrap();
        Ok(())
    }

    fn remove_cargo_project_with_snippet() -> Result<(), String> {
        let current_dir = Path::new(".").canonicalize().unwrap();
        let workspace_path = current_dir.join("tests").join("workspaces").join("temp");
        fs::remove_dir_all(workspace_path).unwrap();
        Ok(())
    }

    fn run_with_cargo_bin_and_snippet(
        snippet: &str,
        plugin_args: &[&str],
    ) -> Result<(String, Option<String>), String> {
        create_cargo_project_with_snippet(snippet).unwrap();
        let result = run_with_cargo_bin("workspaces/temp", None, plugin_args);
        remove_cargo_project_with_snippet().unwrap();
        result
    }

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
        let snippet = r#"
#[cfg(feature = "a")]
fn a() {

    #[cfg(feature = "b")]
    fn b() {}

}
"#;
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

        unimplemented!();

        Ok(())
    }

    #[test]
    fn test_one_in_not() -> Result<(), String> {
        let snippet = r#"
#[cfg(not(feature = "a"))]
fn not_a() {

    #[cfg(feature = "b")]
    fn b() {}

}
"#;
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

        unimplemented!();

        Ok(())
    }

    #[test]
    fn test_one_in_any() -> Result<(), String> {
        let snippet = r#"
#[cfg(any(feature = "a", feature = "b"))]
fn a_b() {

    #[cfg(feature = "c")]
    fn c() {}

}
"#;
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

        unimplemented!();

        Ok(())
    }

    #[test]
    fn test_one_in_all() -> Result<(), String> {
        let snippet = r#"
#[cfg(all(feature = "a", feature = "b"))]
fn a_b() {

    #[cfg(feature = "c")]
    fn c() {}

}
"#;
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

        unimplemented!();

        Ok(())
    }

    // =============================================
    // ==================== NOT ====================
    // =============================================

    #[test]
    fn test_not_in_one() -> Result<(), String> {
        let snippet = r#"
#[cfg(feature = "a")]
fn a() {

    #[cfg(not(feature = "b"))]
    fn not_b() {}

}
"#;
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

        unimplemented!();

        Ok(())
    }

    #[test]
    fn test_not_in_not() -> Result<(), String> {
        let snippet = r#"
#[cfg(not(feature = "a"))]
fn not_a() {

    #[cfg(not(feature = "b"))]
    fn not_b() {}

}
"#;
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

        unimplemented!();

        Ok(())
    }

    #[test]
    fn test_not_in_any() -> Result<(), String> {
        let snippet = r#"
#[cfg(any(feature = "a", feature = "b"))]
fn a_b() {

    #[cfg(not(feature = "c"))]
    fn not_c() {}

}
"#;
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

        unimplemented!();

        Ok(())
    }

    #[test]
    fn test_not_in_all() -> Result<(), String> {
        let snippet = r#"
#[cfg(all(feature = "a", feature = "b"))]
fn a_b() {

    #[cfg(not(feature = "c"))]
    fn not_c() {}

}
"#;
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

        unimplemented!();

        Ok(())
    }

    // =============================================
    // ==================== ALL ====================
    // =============================================

    #[test]
    fn test_all_in_one() -> Result<(), String> {
        let snippet = r#"
#[cfg(feature = "a")]
fn a() {

    #[cfg(all(feature = "b", feature = "c"))]
    fn all_b_c() {}
}
"#;
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

        unimplemented!();

        Ok(())
    }

    #[test]
    fn test_all_in_not() -> Result<(), String> {
        let snippet = r#"
#[cfg(not(feature = "a"))]
fn not_a() {

    #[cfg(all(feature = "b", feature = "c"))]
    fn all_b_c() {}

}
"#;
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

        unimplemented!();
        Ok(())
    }

    #[test]
    fn test_all_in_any() -> Result<(), String> {
        let snippet = r#"
#[cfg(any(feature = "a", feature = "b"))]
fn a_b() {

    #[cfg(all(feature = "c", feature = "d"))]
    fn c_d() {}

}
"#;
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

        unimplemented!();

        Ok(())
    }

    #[test]
    fn test_all_in_all() -> Result<(), String> {
        let snippet = r#"
#[cfg(all(feature = "a", feature = "b"))]
fn a_b() {

    #[cfg(all(feature = "c", feature = "d"))]
    fn c_d() {}

}
"#;
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

        unimplemented!();

        Ok(())
    }

    // =============================================
    // ==================== ANY ====================
    // =============================================

    #[test]
    fn test_any_in_one() -> Result<(), String> {
        let snippet = r#"
#[cfg(feature = "a")]
fn a() {

    #[cfg(any(feature = "b", feature = "c"))]
    fn all_b_c() {}

}
"#;
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

        unimplemented!();

        Ok(())
    }

    #[test]
    fn test_any_in_not() -> Result<(), String> {
        let snippet = r#"
#[cfg(not(feature = "a"))]
fn not_a() {

    #[cfg(any(feature = "b", feature = "c"))]
    fn all_b_c() {}

}
"#;
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

        unimplemented!();
        Ok(())
    }

    #[test]
    fn test_any_in_any() -> Result<(), String> {
        let snippet = r#"
#[cfg(any(feature = "a", feature = "b"))]
fn a_b() {

    #[cfg(any(feature = "c", feature = "d"))]
    fn c_d() {}

}
"#;
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

        unimplemented!();

        Ok(())
    }

    #[test]
    fn test_any_in_all() -> Result<(), String> {
        let snippet = r#"
#[cfg(all(feature = "a", feature = "b"))]
fn a_b() {

    #[cfg(any(feature = "c", feature = "d"))]
    fn c_d() {}

}
"#;
        let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--print-features-dot"])?;

        unimplemented!();

        Ok(())
    }

    // =============================================

    // Advanced tests for the different combinations of cfg attributes
    //
    //          all(any(one not) one) any(all(one not) one)
    // one/not
    // any
    // all
}
