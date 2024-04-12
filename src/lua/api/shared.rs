use anyhow::Result;
use mlua::{Lua, Table};

pub fn register<'lua>(lua: &'lua Lua, root: &'lua Table<'lua>) -> Result<()> {
    root.set("hostname", lua.create_function(hostname)?)?;
    root.set("split", lua.create_function(split)?)?;
    root.set("trim", lua.create_function(trim)?)?;
    root.set("startswith", lua.create_function(startswith)?)?;
    root.set("endswith", lua.create_function(endswith)?)?;

    Ok(())
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
