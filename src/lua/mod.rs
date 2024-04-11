mod api;
mod repl;

use anyhow::{bail, Result};
use mlua::{AsChunk, FromLua, Lua, Table, Value};

pub(crate) use repl::Repl;

static MOD_NAME: &str = "dfim";

// This file is generated by the build script (build.rs). It creates a static array of pairs with
// the form `(modname, bytecode)`. This allows us to easily iterate over the items and load the
// bytecode into the appropriate target.
include!(env!("DFIM_GEN_LUA_BUILTIN"));

pub fn create_state() -> Result<Lua> {
    let lua = Lua::new();

    // scope is needed for borrow lifetime
    {
        let m = create_module(&lua, MOD_NAME)?;
        m.set("version", env!("CARGO_PKG_VERSION"))?;
        m.set("target_triple", env!("VERGEN_CARGO_TARGET_TRIPLE"))?;
        m.set("os_name", std::env::consts::OS)?;
        m.set("os_family", std::env::consts::FAMILY)?;
        m.set("arch", std::env::consts::ARCH)?;
        api::create_native_api(&lua, &m)?;

        lua.globals().set(MOD_NAME, m)?;

        // load compiled bytecode
        for (name, data) in _GEN_BUILTIN {
            let modname = format!("{MOD_NAME}.{name}");
            let value: Value = load_module(&lua, &modname, data)?;
            set_nested_field(&lua, &modname, value)?;
        }
    }

    Ok(lua)
}

/// Creates an empty table in `package.loaded`. If the table already exists, it is returned.
///
/// This function is adapted from [`wezterm`].
///
/// [`wezterm`]: https://github.com/wez/wezterm/blob/e5ac32f297cf3dd8f6ea280c130103f3cac4dddb/config/src/lua.rs#L33-L53
pub fn create_module<'lua>(lua: &'lua Lua, name: &str) -> Result<Table<'lua>> {
    let package: Table = lua.globals().get("package")?;
    let loaded: Table = package.get("loaded")?;
    create_table_in(lua, name, loaded)
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

        let node = create_table_in(lua, part, head.clone())?;
        head = node;
    }
    head.set(tail, value)?;

    Ok(head.get(tail)?)
}

/// Creates a table in the `root` [`Table`]. If it already exists, it is returned.
fn create_table_in<'lua>(lua: &'lua Lua, name: &str, root: Table<'lua>) -> Result<Table<'lua>> {
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
