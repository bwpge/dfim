use std::{error::Error, path::Path, process::Command};

use vergen::EmitBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    EmitBuilder::builder()
        .cargo_target_triple()
        .cargo_debug()
        .cargo_opt_level()
        .git_commit_date()
        .emit()?;

    commit_info();
    Ok(())
}

// adapted from cargo implementation
// see: https://github.com/rust-lang/cargo/blob/7b7af3077bff8d60b7f124189bc9de227d3063a9/build.rs#L50-L79
fn commit_info() {
    if !Path::new(".git").exists() {
        return;
    }
    let output = match Command::new("git")
        .args(["log", "-1", "--format=%H %h"])
        .output()
    {
        Ok(output) if output.status.success() => output,
        _ => return,
    };

    let stdout = String::from_utf8(output.stdout).unwrap();
    for (val, var) in stdout
        .split_whitespace()
        .zip(["CARGO_PKG_GIT_SHA", "CARGO_PKG_GIT_SHA_SHORT"])
    {
        println!("cargo:rustc-env={var}={val}");
    }
}
