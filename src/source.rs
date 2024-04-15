use std::{path::PathBuf, str::FromStr};

use anyhow::bail;
use mlua::{Error as LuaError, FromLua, IntoLua, Lua, Result as LuaResult, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Repo(String),
    Directory(PathBuf),
    File(PathBuf),
}

impl FromStr for Source {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() || s.chars().all(char::is_whitespace) {
            bail!("value must not be empty or whitespace");
        }
        Ok(Self::Repo(s.to_owned()))
    }
}

impl<'lua> FromLua<'lua> for Source {
    fn from_lua(value: Value<'lua>, _lua: &'lua Lua) -> LuaResult<Self> {
        match value {
            Value::String(ref s) => {
                Self::from_str(s.to_str()?).map_err(|e| LuaError::FromLuaConversionError {
                    from: value.type_name(),
                    to: "Source",
                    message: Some(e.to_string()),
                })
            }
            Value::Table(ref t) => {
                if let Ok(s) = t.get::<i32, mlua::String>(1) {
                    return Self::from_lua(Value::String(s), _lua);
                }
                if let Ok(d) = t.get::<&str, String>("dir") {
                    return Ok(Self::Directory(PathBuf::from(d)));
                }
                if let Ok(f) = t.get::<&str, String>("file") {
                    return Ok(Self::File(PathBuf::from(f)));
                }
                Err(LuaError::FromLuaConversionError {
                    from: value.type_name(),
                    to: "Source",
                    message: Some("expected [1], `dir`, or `file` key in table".into()),
                })
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "Source",
                message: Some("expected a table or string value".into()),
            }),
        }
    }
}

impl<'lua> IntoLua<'lua> for Source {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<Value<'lua>> {
        match self {
            Source::Repo(s) => {
                let s = lua.create_string(s)?;
                Ok(Value::String(s))
            }
            Source::Directory(p) => {
                let t = lua.create_table()?;
                let s = p.to_string_lossy().to_string();
                t.set("dir", Value::String(lua.create_string(s)?))?;
                Ok(Value::Table(t))
            }
            Source::File(p) => {
                let t = lua.create_table()?;
                let s = p.to_string_lossy().to_string();
                t.set("file", Value::String(lua.create_string(s)?))?;
                Ok(Value::Table(t))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(lua: &Lua, block: &str) -> Source {
        lua.load(format!("return {block}")).call(()).unwrap()
    }

    #[test]
    fn from_string() {
        let lua = Lua::new();
        let value: Source = call(&lua, "'foo'");
        assert_eq!(value, Source::Repo("foo".into()))
    }

    #[test]
    fn from_table_str() {
        let lua = Lua::new();
        let value: Source = call(&lua, "{'foo'}");
        assert_eq!(value, Source::Repo("foo".into()))
    }

    #[test]
    fn from_table_dir() {
        let lua = Lua::new();
        let value: Source = call(&lua, "{dir = 'foo'}");
        assert_eq!(value, Source::Directory("foo".into()))
    }

    #[test]
    fn from_table_file() {
        let lua = Lua::new();
        let value: Source = call(&lua, "{file = 'foo'}");
        assert_eq!(value, Source::File("foo".into()))
    }

    #[test]
    fn from_table_multi() {
        let lua = Lua::new();
        let value: Source = call(&lua, "{ 'foo', file = 'bar', dir = 'baz' }");
        assert_eq!(value, Source::Repo("foo".into()))
    }
}
