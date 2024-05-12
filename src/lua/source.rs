use std::collections::HashMap;

use anyhow::Result;
use log::{debug, trace, warn};
use mlua::{Error as LuaError, FromLua, Lua, Result as LuaResult, Table, Value};

use crate::{
    lua::consts::registry::{
        flags::{LAYER_CREATED, SOURCES_SET},
        SOURCES,
    },
    source::Source,
};

type SourceMap = HashMap<String, Source>;

pub fn register<'lua>(lua: &'lua Lua, root: &'lua Table<'lua>) -> Result<()> {
    trace!("Registering native module");
    lua.set_named_registry_value(SOURCES, SourceMap::new())?;

    let m = lua.create_table()?;
    m.set("set", lua.create_function(set_sources)?)?;
    m.set("get", lua.create_function(get_sources)?)?;
    root.set("sources", m)?;

    Ok(())
}

fn set_sources(lua: &Lua, value: Table) -> LuaResult<()> {
    if super::get_registry_flag(lua, LAYER_CREATED) {
        return Err(LuaError::runtime(
            "sources cannot be changed after a layer has been created",
        ));
    }
    if super::get_registry_flag(lua, SOURCES_SET) {
        warn!("setting sources multiple times is not recommended (source map is cleared on each call)")
    }

    let mut sources: SourceMap = lua.named_registry_value(SOURCES)?;
    sources.clear();

    let values = value.sequence_values::<Value>();
    for src in values {
        let src = src?;
        let v = Source::from_lua(src.clone(), lua)?;
        let k = if let Value::Table(t) = src {
            if let Ok(name) = t.get::<_, String>("name") {
                name
            } else {
                v.name()
            }
        } else {
            v.name()
        };

        debug!("Registering source: `{k}` => `{v}`");
        if k.is_empty() {
            return Err(LuaError::runtime("source name must not be empty"));
        }
        if sources.contains_key(&k) {
            return Err(LuaError::runtime(format!(
                "source name `{k}` already exists"
            )));
        }

        sources.insert(k, v);
    }

    lua.set_named_registry_value(SOURCES, sources)?;
    super::set_registry_flag(lua, SOURCES_SET, true).map_err(LuaError::runtime)?;

    Ok(())
}

fn get_sources(lua: &Lua, _: ()) -> LuaResult<Table> {
    let sources: SourceMap = lua.named_registry_value(SOURCES)?;
    let table = lua.create_table()?;

    for (k, v) in sources {
        table.set(k, v)?;
    }

    Ok(table)
}
