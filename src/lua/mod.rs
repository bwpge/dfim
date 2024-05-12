mod consts;
mod json;
mod logging;
mod plugin;
mod shared;
mod source;
mod system;
mod traits;

use std::path::Path;

use anyhow::{bail, Result};
use log::{debug, trace};
use mlua::{AsChunk, FromLua, Lua, Table, Value};

use crate::{config::config_dir, path};

static MOD_NAME: &str = env!("CARGO_PKG_NAME");

type RegisterFn = for<'lua> fn(&'lua Lua, &'lua Table<'lua>) -> Result<()>;

const REGISTER_FNS: [RegisterFn; 6] = [
    json::register,
    logging::register,
    plugin::register,
    shared::register,
    source::register,
    system::register,
];

// This file is generated by the build script (build.rs). It creates a const array of pairs with
// the form `(modname, bytecode)`. This allows us to easily iterate over the items and load the
// bytecode into the appropriate targets.
include!(env!("DFIM_GEN_LUA_BUILTIN"));

pub fn create_state() -> Result<Lua> {
    debug!("Creating new Lua state");
    let lua = Lua::new();

    // scope is needed for borrow lifetime
    {
        update_package_path(&lua)?;
        let m = create_module(&lua, MOD_NAME)?;
        m.set("version", env!("CARGO_PKG_VERSION"))?;
        create_native_api(&lua, &m)?;

        lua.globals().set(MOD_NAME, m)?;

        // load compiled bytecode
        for (name, data) in GEN_BUILTIN {
            trace!("Loading module bytecode `{name}`");
            let modname = format!("{MOD_NAME}.{name}");
            let value: Value = load_module(&lua, &modname, data)?;
            set_nested_field(&lua, &modname, value)?;
        }
    }

    Ok(lua)
}

/// Sets the `package.path` to include application-specific directories.
fn update_package_path(lua: &Lua) -> Result<()> {
    let package: Table = lua.globals().get("package")?;
    let package_path: String = package.get("path")?;

    fn lua_path(path: &Path) -> Vec<String> {
        let mut v = Vec::with_capacity(2);
        let path = path.display();
        v.push(format!("{path}{}", path!(/ "?.lua")));
        v.push(format!("{path}{}", path!(/ "?", "init.lua")));
        v
    }

    let paths: Vec<String> = lua_path(config_dir())
        .into_iter()
        .chain(package_path.split_terminator(';').map(ToOwned::to_owned))
        .collect();

    debug!("Setting package path:\n{paths:#?}");
    package.set("path", paths.join(";"))?;

    Ok(())
}

/// Runs the `register` function for native lua API modules.
fn create_native_api<'lua>(lua: &'lua Lua, root: &'lua Table<'lua>) -> Result<()> {
    debug!("Registering native API modules");
    for func in REGISTER_FNS {
        func(lua, root)?;
    }

    Ok(())
}

/// Creates an empty table in `package.loaded`. If the table already exists, it is returned.
///
/// This function is adapted from [`wezterm`].
///
/// [`wezterm`]: https://github.com/wez/wezterm/blob/e5ac32f297cf3dd8f6ea280c130103f3cac4dddb/config/src/lua.rs#L33-L53
fn create_module<'lua>(lua: &'lua Lua, name: &str) -> Result<Table<'lua>> {
    let package: Table = lua.globals().get("package")?;
    let loaded: Table = package.get("loaded")?;
    create_table_in(lua, name, &loaded)
}

/// Sets a nested field with the form `foo.bar.baz` by splitting on `.` separators.
///
/// Each intermediate split is set to a table if it does not exist, and the final split is set to
/// the provided `value`.
pub(crate) fn set_nested_field<'lua>(
    lua: &'lua Lua,
    name: &str,
    value: Value<'lua>,
) -> Result<Value<'lua>> {
    let mut parts = name.split('.').rev().collect::<Vec<_>>();
    if parts.is_empty() {
        bail!("module must have a root");
    }

    let tail = parts.remove(0);
    if parts.is_empty() {
        lua.globals().set(tail, value)?;
        return Ok(lua.globals().get(tail)?);
    }

    let root = create_module(lua, parts.pop().unwrap())?;
    let mut head = root;

    while let Some(part) = parts.pop() {
        if part.is_empty() {
            bail!("module parts must not be empty");
        }

        let node = create_table_in(lua, part, &head)?;
        head = node;
    }
    head.set(tail, value)?;

    Ok(head.get(tail)?)
}

/// Creates a table in the `root` [`Table`]. If it already exists, it is returned.
fn create_table_in<'lua>(lua: &'lua Lua, name: &str, root: &Table<'lua>) -> Result<Table<'lua>> {
    let module = root.get(name)?;
    match module {
        Value::Nil => {
            let m = lua.create_table()?;
            root.set(name, m.clone())?;
            Ok(m)
        }
        Value::Table(m) => Ok(m),
        other => anyhow::bail!(
            "cannot create table `{name}`, value exists with type `{}`",
            other.type_name()
        ),
    }
}

/// Creates a module by loading `data` with [`Lua::load_from_function`].
///
/// Generally speaking, modules are expected to return a [`Table`], but this is not strictly
/// required. The output from `data` is stored in `package.loaded["name"]`.
pub(crate) fn load_module<'lua, 'c, C, T>(lua: &'lua Lua, name: &str, data: C) -> Result<T>
where
    C: AsChunk<'lua, 'c>,
    T: FromLua<'lua>,
{
    let f = lua.load(data).into_function()?;
    Ok(lua.load_from_function(name, f)?)
}

pub(crate) fn get_registry_flag(lua: &Lua, key: &str) -> bool {
    lua.named_registry_value::<bool>(key).unwrap_or_default()
}

pub(crate) fn set_registry_flag(lua: &Lua, key: &str, value: bool) -> Result<()> {
    if lua.named_registry_value::<bool>(key).is_err() {
        bail!("registry already contains `{key}` and is not a boolean value")
    }

    lua.set_named_registry_value(key, value)?;
    Ok(())
}
