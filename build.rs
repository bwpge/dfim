use std::{
    error::Error,
    ffi::OsStr,
    fmt,
    fs::File,
    io::Write,
    path::{Component, Path, PathBuf},
    process::Command,
};

use mlua::Lua;
use vergen::EmitBuilder;

const LUA_SRC_DIR: &str = "runtime";

fn main() -> Result<(), Box<dyn Error>> {
    EmitBuilder::builder()
        .cargo_target_triple()
        .cargo_debug()
        .cargo_opt_level()
        .git_commit_date()
        .emit()?;

    commit_info();
    compile_lua();
    wsl_info();
    Ok(())
}

/// Emits a `cargo:rustc-env` line.
///
/// Neither `key` nor `value` are checked, so the caller must ensure they produce a valid
/// environment mapping for `rustc`.
fn set_rustc_var<K: fmt::Display, V: fmt::Display>(key: K, value: V) {
    println!("cargo:rustc-env={key}={value}");
}

/// Runs a command with the given `args` and gets `stdout` as a [`String`].
fn get_command_stdout<I, S>(prog: S, args: I) -> Option<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = match Command::new(prog).args(args).output() {
        Ok(output) if output.status.success() => output,
        _ => return None,
    };

    // we want a compiler error if we don't get valid utf-8 from command,
    // things get too weird if we're trying to handle utf-8, -16, -32, etc.
    Some(String::from_utf8(output.stdout).unwrap())
}

/// Generates commit information, similar to the implementation of [`cargo`].
///
/// [`cargo`]: https://github.com/rust-lang/cargo/blob/7b7af3077bff8d60b7f124189bc9de227d3063a9/build.rs#L50-L79
fn commit_info() {
    if !Path::new(".git").exists() {
        return;
    }

    let stdout = get_command_stdout("git", ["log", "-1", "--format=%H %h"]).unwrap();
    let mut parts = stdout.split_whitespace();
    let mut next = || parts.next().unwrap();
    set_rustc_var("DFIM_GIT_SHA", next());
    set_rustc_var("DFIM_GIT_SHA_SHORT", next());
}

/// Compiles lua modules in [`LUA_SRC_DIR`] and generates a source file. The path to the file will
/// stored in `DFIM_GEN_LUA_BUILTIN`, which can be used later by [`include`].
fn compile_lua() {
    println!("cargo:rerun-if-changed={LUA_SRC_DIR}/");
    let lua = Lua::new();
    let src_dir = format!("{LUA_SRC_DIR}/**/*.lua");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let mut compiled = vec![];

    for path in glob::glob(&src_dir).unwrap() {
        // get module name from path
        let src = path.unwrap();
        let modname = get_module_name(&src);

        // compile
        let func = lua.load(src.clone()).into_function().unwrap();
        let bc = func.dump(true);

        // write bytes to file
        let f = out_dir.join(format!("{modname}.ljbc",));
        std::fs::write(&f, bc).unwrap();
        compiled.push((modname, f));
    }

    // generate rust source file for bytecode
    let gen_path = out_dir.join("gen_lua_builtin.rs");
    set_rustc_var("DFIM_GEN_LUA_BUILTIN", gen_path.display());

    let mut file = File::create(&gen_path).unwrap();
    file.write_all(
        format!(
            "/// Compiled bytecode for builtin lua modules.\n\
            ///\n\
            /// Pairs have the form `(modname, bytecode)`.\n\
            const GEN_BUILTIN: [(&str, &[u8]); {}] = [\n",
            compiled.len()
        )
        .as_bytes(),
    )
    .unwrap();
    for (modname, f) in compiled {
        let bytes = std::fs::read(f)
            .unwrap()
            .into_iter()
            .map(|b| format!("0x{b:02x}"))
            .collect::<Vec<_>>()
            .join(",");
        file.write_all(format!("    (\"{modname}\", &[").as_bytes())
            .unwrap();
        file.write_all(bytes.as_bytes()).unwrap();
        file.write_all(b"]),\n").unwrap();
    }

    file.write_all(b"];").unwrap();
    file.flush().unwrap();
    drop(file);
}

/// Converts path parts to a lua module name e.g., `foo/bar/baz` becomes `foo.bar.baz`.
///
/// Lua module names do not have any particular identifier requirements, but this function enforces
/// some basic restrictions like starting with a letter and not containing punctuation aside from
/// `-` and `_`.
fn get_module_name<P: AsRef<Path>>(path: P) -> String {
    let path = path.as_ref().strip_prefix(LUA_SRC_DIR).unwrap();
    let mut parts = vec![];
    for part in path.components() {
        if let Component::Normal(p) = part {
            let s = p.to_string_lossy();
            let s = s.strip_suffix(".lua").unwrap_or(&s).to_owned();
            assert!(!s.is_empty(), "module part must not be empty");
            assert!(
                s.chars().next().unwrap().is_alphabetic(),
                "module part must start with a letter"
            );
            assert!(
                s.chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_'),
                "module name must only contain alphanumeric, -, or _"
            );
            parts.push(s);
        }
    }

    parts.join(".")
}

/// Sets a rustc environment variable if running on WSL.
///
/// The value should not be used, only checked if set e.g., `option_env!("DFIM_WSL").is_some()`.
fn wsl_info() {
    if std::env::consts::OS != "linux" {
        return;
    }

    if let Some(stdout) = get_command_stdout("uname", ["-a"]) {
        if stdout.to_lowercase().contains("microsoft") {
            set_rustc_var("DFIM_WSL", 1);
        }
    }
}
