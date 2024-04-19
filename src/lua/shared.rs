use std::path::PathBuf;

use anyhow::Result;
use log::trace;
use mlua::{Error as LuaError, Lua, Result as LuaResult, Table, Value};

pub fn register<'lua>(lua: &'lua Lua, root: &'lua Table<'lua>) -> Result<()> {
    trace!("Registering native module");
    root.set("split", lua.create_function(split)?)?;
    root.set("trim", lua.create_function(trim)?)?;
    root.set("startswith", lua.create_function(startswith)?)?;
    root.set("endswith", lua.create_function(endswith)?)?;
    root.set("joinpath", lua.create_function(joinpath)?)?;

    Ok(())
}

/// Lua function for splitting a string by another string.
fn split(
    _: &Lua,
    (value, pat, remove_empty): (String, String, Option<bool>),
) -> LuaResult<Vec<String>> {
    let remove_empty = remove_empty.unwrap_or_default();
    Ok(value
        .split(&pat)
        .filter_map(|s| {
            if remove_empty && s.is_empty() {
                None
            } else {
                Some(s.to_owned())
            }
        })
        .collect())
}

/// Lua function for trimming whitespace from a string.
fn trim(lua: &Lua, value: String) -> LuaResult<mlua::String> {
    lua.create_string(value.trim())
}

/// Lua function for checking if a string starts with a prefix.
fn startswith(_: &Lua, (value, pat): (String, String)) -> LuaResult<bool> {
    Ok(value.starts_with(&pat))
}

/// Lua function for checking if a string ends with a suffix.
fn endswith(_: &Lua, (value, pat): (String, String)) -> LuaResult<bool> {
    Ok(value.ends_with(&pat))
}

/// Lua function for joining path parts.
///
/// Useful for working with OS path semantics in Lua, rather than basic strings.
fn joinpath(_: &Lua, mut value: mlua::MultiValue) -> LuaResult<String> {
    let front = value
        .pop_front()
        .ok_or_else(|| LuaError::runtime("at least one path part must not be provided"))?;
    let mut path = lua_value_to_pathbuf(front)?;

    while let Some(part) = value.pop_front() {
        let p = lua_value_to_pathbuf(part)?;
        path = path.join(p);
    }

    Ok(path.to_string_lossy().to_string())
}

fn lua_value_to_pathbuf(value: Value) -> LuaResult<PathBuf> {
    match value {
        // we don't handle invalid utf-8 for paths here. although unix paths
        // can have arbitrary bytes, it makes this code way more complicated
        // for minimal benefit.
        mlua::Value::String(s) => Ok(PathBuf::from(s.to_str()?)),
        _ => Err(LuaError::FromLuaConversionError {
            from: value.type_name(),
            to: "Path",
            message: Some("expected a string value".into()),
        }),
    }
}
