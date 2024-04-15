use anyhow::Result;
use log::trace;
use mlua::{Lua, MultiValue, Result as LuaResult, Table, Value};

use crate::{config::plugin_dir, path, pathsep};

pub fn register<'lua>(lua: &'lua Lua, _: &'lua Table) -> Result<()> {
    trace!("Setting plugin package searcher");
    let package: Table = lua.globals().get("package")?;
    let searchers: Table = package.get("searchers")?;
    searchers.set(searchers.len()? + 1, lua.create_function(search_plugins)?)?;

    Ok(())
}

fn search_plugins(lua: &Lua, modname: String) -> LuaResult<MultiValue> {
    let plugin_dir = plugin_dir();
    trace!("Searching for plugin module `{modname}`");

    if !plugin_dir.is_dir() {
        let reason = lua.create_string("\n\tno dfim plugin directory")?;
        return Ok(MultiValue::from_iter([Value::String(reason)]));
    }

    let mut count = 0;
    for entry in std::fs::read_dir(plugin_dir)? {
        let entry = entry?.path();
        if !entry.is_dir() {
            continue;
        }
        count += 1;

        let dir = entry.join("lua");
        let name = modname.replace('.', pathsep!());

        for suffix in [".lua", path!(/ "init.lua")] {
            let p = dir.join(format!("{name}{suffix}"));
            trace!("Searching for {}", p.display());

            if p.is_file() {
                let loader = lua.load(p.clone()).into_function()?;
                let src = lua.create_string(p.to_string_lossy().to_string())?;
                return Ok(MultiValue::from_iter([
                    Value::Function(loader),
                    Value::String(src),
                ]));
            }
        }
    }

    let reason = lua.create_string(format!(
        "\n\tno dfim plugins contain module '{modname}' ({count} searched)"
    ))?;
    Ok(MultiValue::from_iter([Value::String(reason)]))
}
