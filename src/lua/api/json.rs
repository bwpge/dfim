use std::{fs::File, io::BufReader};

use anyhow::Result;
use mlua::{Error as LuaError, IntoLua, Lua, Result as LuaResult, Table, Value};
use serde_json::Value as JValue;

pub fn register<'lua>(lua: &'lua Lua, root: &'lua Table<'lua>) -> Result<()> {
    let m = lua.create_table()?;
    m.set("encode", lua.create_function(to_json)?)?;
    m.set("decode", lua.create_function(from_json)?)?;
    m.set("decode_file", lua.create_function(from_json_file)?)?;
    root.set("json", m)?;

    Ok(())
}

fn to_json(_: &Lua, (value, pretty): (Value, bool)) -> LuaResult<String> {
    let s = if pretty {
        serde_json::to_string_pretty(&value).map_err(LuaError::external)?
    } else {
        serde_json::to_string(&value).map_err(LuaError::external)?
    };

    Ok(s)
}

fn from_json(lua: &Lua, value: String) -> LuaResult<Value> {
    let value: JValue = serde_json::from_str(&value).map_err(LuaError::external)?;
    from_json_impl(lua, value)
}

fn from_json_file(lua: &Lua, (value, buffered): (String, bool)) -> LuaResult<Value> {
    if buffered {
        let reader = BufReader::new(File::open(value)?);
        let value: JValue = serde_json::from_reader(reader).map_err(LuaError::external)?;
        from_json_impl(lua, value)
    } else {
        from_json(lua, std::fs::read_to_string(value)?)
    }
}

// adapted from wezterm, see:
// https://github.com/wez/wezterm/blob/e5ac32f297cf3dd8f6ea280c130103f3cac4dddb/lua-api-crates/serde-funcs/src/lib.rs
fn from_json_impl(lua: &Lua, value: JValue) -> LuaResult<Value> {
    Ok(match value {
        JValue::Null => Value::Nil,
        JValue::Bool(b) => Value::Boolean(b),
        JValue::Number(n) => {
            if let Some(x) = n.as_i64() {
                Value::Integer(x)
            } else if let Some(x) = n.as_f64() {
                Value::Number(x)
            } else {
                return Err(LuaError::external(
                    "value `{n:?}` is not representable as i64 or f64",
                ));
            }
        }
        JValue::String(s) => s.into_lua(lua)?,
        JValue::Array(arr) => {
            let tbl = lua.create_table_with_capacity(arr.len(), 0)?;
            for (i, val) in arr.into_iter().enumerate() {
                tbl.set(i + 1, from_json_impl(lua, val)?)?;
            }
            Value::Table(tbl)
        }
        JValue::Object(map) => {
            let tbl = lua.create_table_with_capacity(0, map.len())?;
            for (key, val) in map.into_iter() {
                tbl.set(key, from_json_impl(lua, val)?)?;
            }
            Value::Table(tbl)
        }
    })
}
