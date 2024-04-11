use std::process::Command;

use anyhow::Result;
use mlua::{Lua, LuaSerdeExt, Table, Value};
use serde::{Deserialize, Serialize};

pub fn register<'lua>(lua: &'lua Lua, root: &'lua Table<'lua>) -> Result<()> {
    root.set("system", lua.create_function(run)?)?;

    Ok(())
}

#[derive(Default, Serialize, Deserialize)]
struct RunOptions {
    #[serde(default)]
    args: Option<Vec<String>>,
    // TODO: add other command options
}

pub(crate) fn run<'lua>(
    lua: &'lua Lua,
    (prog, opts): (String, Value),
) -> mlua::Result<Table<'lua>> {
    let opts: RunOptions = lua.from_value(opts).unwrap_or_default();

    let args = opts.args.unwrap_or_default();
    let mut cmd = Command::new(prog);
    if !args.is_empty() {
        cmd.args(args);
    }

    let output = cmd.output()?;
    let tbl = lua.create_table_with_capacity(0, 3)?;
    tbl.set("code", output.status.code())?;
    tbl.set("stdout", lua.create_string(&output.stdout)?)?;
    tbl.set("stderr", lua.create_string(&output.stderr)?)?;

    Ok(tbl)
}
