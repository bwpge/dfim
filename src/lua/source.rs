use anyhow::Result;
use log::trace;
use mlua::{IntoLua, Lua, Result as LuaResult, Table, UserData, UserDataMethods, Value};

use crate::source::Source;

#[derive(Debug, Default)]
struct Sources {
    values: Vec<Source>,
}

impl UserData for Sources {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("add", add);
        methods.add_method_mut("remove", remove);
        methods.add_method("get", get);
        methods.add_method("contains", contains);
    }
}

pub fn register<'lua>(_: &'lua Lua, root: &'lua Table<'lua>) -> Result<()> {
    trace!("Setting source userdata");
    root.set("sources", Sources::default())?;
    Ok(())
}

/// Adds an item to the source list if it does not already exist.
fn add(_: &Lua, this: &mut Sources, value: Source) -> LuaResult<()> {
    // PERF: checking contains each time is not ideal, but source lists really
    // should not get that long. if it becomes a problem we can refactor.
    if !this.values.contains(&value) {
        this.values.push(value);
    }
    Ok(())
}

/// Tries to remove an item from the source list. Returns `true` if the item was removed, `false`
/// otherwise.
fn remove(_: &Lua, this: &mut Sources, value: Source) -> LuaResult<bool> {
    // PERF: same note as with `add`. this really shouldn't be a problem but we
    // can refactor if the performance becomes an issue.
    if let Some(idx) = this.values.iter().position(|x| x == &value) {
        this.values.remove(idx);
        return Ok(true);
    }

    Ok(false)
}

/// Returns the source list as a table.
fn get<'lua>(lua: &'lua Lua, this: &Sources, _: ()) -> LuaResult<Value<'lua>> {
    this.values.clone().into_lua(lua)
}

/// Returns whether or not the source list contains `value`.
fn contains(_: &Lua, this: &Sources, value: Source) -> LuaResult<bool> {
    Ok(this.values.contains(&value))
}
