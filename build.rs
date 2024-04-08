use std::{
    error::Error,
    fs::File,
    io::Write,
    path::{Component, Path, PathBuf},
    process::Command,
};

use mlua::Lua;
use vergen::EmitBuilder;

const LUA_SRC_DIR: &str = "lua";

fn main() -> Result<(), Box<dyn Error>> {
    EmitBuilder::builder()
        .cargo_target_triple()
        .cargo_debug()
        .cargo_opt_level()
        .git_commit_date()
        .emit()?;

    commit_info();
    compile_lua().unwrap();
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

fn compile_lua() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed={LUA_SRC_DIR}/");
    let lua = Lua::new();
    let src_dir = format!("{LUA_SRC_DIR}/**/*.lua");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    let mut compiled = vec![];

    for path in glob::glob(&src_dir)? {
        // figure out path stuff
        let src = path?;
        let modname = get_module_name(&src);

        // compile
        let func = lua.load(src.clone()).into_function()?;
        let bc = func.dump(true);

        // write bytes to file
        let fout = out_dir.join(format!("{modname}.ljbc",));
        std::fs::write(&fout, bc)?;
        compiled.push((modname, fout));
    }

    // generate rust source file for bytecode
    let gen_path = out_dir.join("gen_lua_builtin.rs");
    println!(
        "cargo:rustc-env=DFIM_GEN_LUA_BUILTIN={}",
        gen_path.display()
    );

    let mut genf = File::create(&gen_path)?;
    genf.write_all(
        format!(
            "/// Compiled bytecode for builtin lua modules.\n\
            ///\n\
            /// Pairs have the form `(modname, bytecode)`.\n\
            static _GEN_BUILTIN: [(&str, &[u8]); {}] = [\n",
            compiled.len()
        )
        .as_bytes(),
    )?;
    let suffix = b"]),\n";
    for (modname, f) in compiled {
        let bytes = std::fs::read(f)
            .unwrap()
            .into_iter()
            .map(|b| format!("0x{b:02x}"))
            .collect::<Vec<_>>()
            .join(",");
        genf.write_all(format!("    (\"{modname}\", &[").as_bytes())?;
        genf.write_all(bytes.as_bytes())?;
        genf.write_all(suffix)?;
    }

    genf.write_all(b"];")?;
    genf.flush()?;
    drop(genf);

    Ok(())
}

fn get_module_name(path: &Path) -> String {
    let path = path.strip_prefix(LUA_SRC_DIR).unwrap();
    let mut parts = vec![];
    for part in path.components() {
        match part {
            Component::Normal(p) => {
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
            _ => (),
        }
    }

    parts.join(".")
}
