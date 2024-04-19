use std::{collections::HashMap, path::PathBuf, process::Command};

use anyhow::Result;
use log::trace;
use mlua::{Error as LuaError, Lua, LuaSerdeExt, Table, Value};
use serde::Deserialize;

pub fn register<'lua>(lua: &'lua Lua, root: &'lua Table<'lua>) -> Result<()> {
    trace!("Registering native module");
    root.set("spawn", lua.create_function(run)?)?;

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

pub(crate) fn run<'lua>(
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
