use anyhow::Result;
use log::{debug, error, info, log, trace, warn, Level};
use mlua::{Lua, Table};

pub fn register<'lua>(lua: &'lua Lua, root: &'lua Table<'lua>) -> Result<()> {
    trace!("Registering native module");
    root.set("trace", lua.create_function(trace)?)?;
    root.set("debug", lua.create_function(debug)?)?;
    root.set("info", lua.create_function(info)?)?;
    root.set("warn", lua.create_function(warn)?)?;
    root.set("error", lua.create_function(error)?)?;

    let levels = lua.create_table()?;
    levels.set("ERROR", Level::Error as usize)?;
    levels.set("WARN", Level::Warn as usize)?;
    levels.set("INFO", Level::Info as usize)?;
    levels.set("DEBUG", Level::Debug as usize)?;
    levels.set("TRACE", Level::Trace as usize)?;

    let m = lua.create_table()?;
    m.set("levels", levels)?;

    let mt = lua.create_table()?;
    mt.set(
        "__call",
        lua.create_function(
            |lua, (_, msg, level, opts): (mlua::Value, String, Option<usize>, Option<Table>)| {
                log(lua, (msg, level, opts))
            },
        )?,
    )?;
    m.set_metatable(Some(mt));
    root.set("log", m)?;

    Ok(())
}

fn fmt_target(opts: Option<Table>) -> String {
    let target = opts
        .and_then(|t| t.get::<&str, String>("target").ok())
        .unwrap_or_default();
    let sep = if target.is_empty() { "" } else { "::" };

    format!("LUA{sep}{target}")
}

fn log(
    _: &Lua,
    (message, level, opts): (String, Option<usize>, Option<mlua::Table>),
) -> mlua::Result<()> {
    let level = match level.unwrap_or(3) {
        x if x == Level::Error as usize => Level::Error,
        x if x == Level::Warn as usize => Level::Warn,
        x if x == Level::Info as usize => Level::Info,
        x if x == Level::Debug as usize => Level::Debug,
        x if x == Level::Trace as usize => Level::Trace,
        _ => Level::Info,
    };
    let target = fmt_target(opts);
    log!(target: &target, level, "{message}");

    Ok(())
}

macro_rules! impl_level_fn {
    ($name:ident) => {
        fn $name(_: &Lua, (message, opts): (String, Option<mlua::Table>)) -> mlua::Result<()> {
            let target = fmt_target(opts);
            $name!(target: &target, "{message}");

            Ok(())
        }
    };
}

impl_level_fn!(trace);
impl_level_fn!(debug);
impl_level_fn!(info);
impl_level_fn!(warn);
impl_level_fn!(error);
