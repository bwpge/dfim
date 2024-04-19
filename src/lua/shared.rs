use anyhow::Result;
use log::trace;
use mlua::{Lua, Table};

pub fn register<'lua>(lua: &'lua Lua, root: &'lua Table<'lua>) -> Result<()> {
    trace!("Registering native module");
    root.set("split", lua.create_function(split)?)?;
    root.set("trim", lua.create_function(trim)?)?;
    root.set("startswith", lua.create_function(startswith)?)?;
    root.set("endswith", lua.create_function(endswith)?)?;

    Ok(())
}

/// Lua function for splitting a string by another string.
fn split(
    _: &Lua,
    (value, pat, remove_empty): (String, String, Option<bool>),
) -> mlua::Result<Vec<String>> {
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
fn trim(lua: &Lua, value: String) -> mlua::Result<mlua::String> {
    lua.create_string(value.trim())
}

/// Lua function for checking if a string starts with a prefix.
fn startswith(_: &Lua, (value, pat): (String, String)) -> mlua::Result<bool> {
    Ok(value.starts_with(&pat))
}

/// Lua function for checking if a string ends with a suffix.
fn endswith(_: &Lua, (value, pat): (String, String)) -> mlua::Result<bool> {
    Ok(value.ends_with(&pat))
}
