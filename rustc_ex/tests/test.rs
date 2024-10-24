use std::fs;
use std::process::Command;
use std::sync::Once;
use std::{env, path::Path};

const PLUGIN_NAME: &str = "ast-visitor";
static SETUP: Once = Once::new();

fn run(workspace: &str) -> Result<String, String> {
    // Install the plugin
    let root_dir = env::temp_dir().join("ast-visitor");
    let current_dir = Path::new(".").canonicalize().unwrap();
    println!("root_dir: {}", root_dir.display());
    println!("current_dir: {}", current_dir.display());
    SETUP.call_once(|| {
        let mut cargo_cmd = Command::new("cargo");
        cargo_cmd.args([
            "install",
            "--path",
            ".",
            "--debug",
            "--locked",
            "--root",
        ]);
        cargo_cmd.arg(&root_dir);
        cargo_cmd.current_dir(&current_dir);
        let status = cargo_cmd.status().unwrap();
        if !status.success() {
            panic!("Failed to install ast-visitor");
        }
    });

    // Run the plugin
    let path = format!("{}:{}", root_dir.join("bin").display(), env::var("PATH").unwrap_or_default());
    let workspace_path = current_dir.join("tests").join(workspace);
    let mut cargo_cmd = Command::new("cargo");
    println!("workspace_path: {}", workspace_path.display());
    println!("path: {}", path);
    cargo_cmd.arg(PLUGIN_NAME);
    cargo_cmd.env("PATH", path);
    cargo_cmd.current_dir(&workspace_path);

    let _ = fs::remove_dir_all(workspace_path.join("target"));
    let output = cargo_cmd.output().unwrap();
    // assert!(output.status.success());

    Ok(String::from_utf8(output.stdout).unwrap())
}

#[test]
fn first() -> Result<(), String> {
    let output = run("workspaces/first");
    println!("TEST: {}", output?);
    Ok(())
}
