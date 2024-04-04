use anyhow::Result;
use mlua::{AsChunk, Lua, Table, Value};

static MOD_NAME: &str = "dfim";

// TODO: builtin modules should be compiled, e.g.:
// luajit -b -s source_file -: writes bytecode to stdout
static BUILTIN: &[(&str, &str)] = &[("inspect", include_str!("../lua/inspect.lua"))];

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
        lua.globals().set(MOD_NAME, m)?;

        // preload builtins
        for &(name, data) in BUILTIN {
            preload_module(&lua, name, data)?;
            lua.load(format!("{MOD_NAME}.{name}=require('{name}')"))
                .exec()?;
        }
    }

    Ok(lua)
}

/// Creates an empty table in `package.loaded`. If the table already exists, it is returned.
///
/// This function is adapted from [`wezterm`].
///
/// [`wezterm`]: <https://github.com/wez/wezterm/blob/e5ac32f297cf3dd8f6ea280c130103f3cac4dddb/config/src/lua.rs#L33-L53>
pub fn create_module<'lua>(lua: &'lua Lua, name: &str) -> Result<Table<'lua>> {
    let package: Table = lua.globals().get("package")?;
    let loaded: Table = package.get("loaded")?;

    let module = loaded.get(name)?;
    match module {
        Value::Nil => {
            let m = lua.create_table()?;
            loaded.set(name, m.clone())?;
            Ok(m)
        }
        Value::Table(m) => Ok(m),
        other => anyhow::bail!(
            "cannot create module `{name}`, value exists with type `{}`",
            other.type_name()
        ),
    }
}

/// Stores the specified `data` as a preload module with the given `name`.
///
/// This module can later be loaded with `required("name")`, which will be returned by calling the
/// value set in `package.preload["name"]`.
pub fn preload_module<'lua, 'c>(
    lua: &'lua Lua,
    name: &str,
    data: impl AsChunk<'lua, 'c>,
) -> Result<()> {
    let package: Table = lua.globals().get("package")?;
    let preload: Table = package.get("preload")?;
    let f = lua.load(data).into_function()?;

    Ok(preload.set(name, f)?)
}
