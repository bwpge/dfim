use std::{collections::HashMap, path::PathBuf, process::Command};

use anyhow::Result;
use log::trace;
use mlua::{Error as LuaError, Lua, LuaSerdeExt, Table, Value};
use serde::Deserialize;

pub fn register<'lua>(lua: &'lua Lua, root: &'lua Table<'lua>) -> Result<()> {
    trace!("Registering native module");
    root.set("target_triple", env!("VERGEN_CARGO_TARGET_TRIPLE"))?;
    root.set("os_name", std::env::consts::OS)?;
    root.set("os_family", std::env::consts::FAMILY)?;
    root.set("arch", std::env::consts::ARCH)?;
    root.set("is_wsl", option_env!("DFIM_WSL").is_some())?;
    root.set("spawn", lua.create_function(spawn)?)?;
    root.set("hostname", lua.create_function(hostname)?)?;
    root.set("cwd", lua.create_function(cwd)?)?;

    Ok(())
}

#[derive(Default, Deserialize)]
struct RunOptions {
    #[serde(default)]
    cwd: Option<PathBuf>,
    #[serde(default)]
    env: HashMap<String, String>,
    #[serde(default)]
    clear_env: Option<bool>,
}

pub(crate) fn spawn<'lua>(
    lua: &'lua Lua,
    (mut args, opts): (Vec<String>, Option<Table>),
) -> mlua::Result<Table<'lua>> {
    if args.is_empty() {
        return Err(LuaError::RuntimeError(
            "cannot execute empty command".into(),
        ));
    }

    let prog = args.remove(0);
    let opts = opts
        .map(|t| lua.from_value(Value::Table(t)))
        .unwrap_or_else(|| Ok(RunOptions::default()))?;

    let mut cmd = Command::new(prog);
    if !args.is_empty() {
        cmd.args(args);
    }
    if let Some(cwd) = opts.cwd {
        cmd.current_dir(cwd);
    }
    if opts.clear_env.unwrap_or_default() {
        cmd.env_clear();
    }
    cmd.envs(&opts.env);
    let output = cmd
        .output()
        .map_err(|e| LuaError::RuntimeError(e.to_string()))?;

    let tbl = lua.create_table()?;
    tbl.set("success", output.status.success())?;
    tbl.set("code", output.status.code())?;
    tbl.set("stdout", lua.create_string(output.stdout)?)?;
    tbl.set("stderr", lua.create_string(output.stderr)?)?;

    Ok(tbl)
}

/// Lua function to get the system hostname.
///
/// This function is adapted from [`wezterm`].
///
/// [`wezterm`]: https://github.com/wez/wezterm/blob/e5ac32f297cf3dd8f6ea280c130103f3cac4dddb/config/src/lua.rs#L427-L433
fn hostname(_: &Lua, _: ()) -> mlua::Result<String> {
    hostname::get()
        .map_err(mlua::Error::external)?
        .to_str()
        .map(|s| s.to_owned())
        .ok_or_else(|| mlua::Error::external("hostname is not valid utf-8"))
}

/// Returns the current working directory as a string.
fn cwd(_: &Lua, _: ()) -> mlua::Result<String> {
    match std::env::current_dir() {
        Ok(p) => Ok(p.to_string_lossy().to_string()),
        Err(e) => Err(LuaError::RuntimeError(e.to_string())),
    }
}
