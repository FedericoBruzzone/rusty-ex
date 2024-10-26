use std::fs;
use std::process::Command;
use std::sync::Once;
use std::{env, path::Path};

const PLUGIN_NAME: &str = "rustc-ex";
static SETUP: Once = Once::new();

/// Run the plugin with the `cargo` command
///
/// This function will install the plugin (cargo-rustc-ex binary) in a temporary directory and run it with the `cargo` command.
/// The plugin will be installed only once.
///
/// # Arguments
/// * `cargo_project_name` - The name of the cargo project in the `tests` directory. E.g. `workspaces/first`
/// * `expected_outout_name` - The name of the file containing the expected output in the cargo project directory. E.g. `expected_output.stdout`
/// * `plugin_args` - The arguments to pass to the plugin
fn run_with_cargo_bin(
    cargo_project_name: &str,
    expected_outout_name: Option<&str>,
    plugin_args: &[&str],
) -> Result<(String, Option<String>), String> {
    // Install the plugin
    let root_dir = env::temp_dir().join("ast-visitor");
    let current_dir = Path::new(".").canonicalize().unwrap();
    SETUP.call_once(|| {
        let mut cargo_cmd = Command::new("cargo");
        cargo_cmd.args(["install", "--path", ".", "--debug", "--locked", "--root"]);
        cargo_cmd.arg(&root_dir);
        cargo_cmd.current_dir(&current_dir);
        let status = cargo_cmd.status().unwrap();
        if !status.success() {
            panic!("Failed to install ast-visitor");
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
use pretty_assertions::assert_eq;

#[test]
fn test_first_dotfile_same_output() -> Result<(), String> {
    let (output, expected_output) = run_with_cargo_bin(
        "workspaces/first",
        Some("expected_output.stdout"),
        &["--print-dot"],
    )?;
    assert_eq!(output, expected_output.unwrap()); // Here, unwrap is "safe" because we want to panic if the expected output file is not present
    Ok(())
}

#[test]
fn test_first_dot_file_contains_features() -> Result<(), String> {
    let (output, _) = run_with_cargo_bin("workspaces/first", None, &["--print-dot"])?;

    for feature in &["aa", "bb", "bb", "cc", "dd", "ee", "ff"] {
        assert!(output.contains(feature));
    }

    Ok(())
}
