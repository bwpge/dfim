use mlua::{Error as LuaError, IntoLuaMulti, Lua, Result, Table, Value};

/// A type that can be represented by a [`Value`] variant, or a [`Value::Function`] returning the
/// correct variant.
pub(crate) trait LuaFlexValue<'lua> {
    /// Tries to return the correct [`Value`] variant if the Lua type matches.
    ///
    /// If the `value` is a [`Value::Function`], it is called with `args` and tries to convert the
    /// return value to this type.
    fn flex_value<A>(lua: &'lua Lua, value: Value<'lua>, args: A) -> Result<Self>
    where
        Self: Sized,
        A: IntoLuaMulti<'lua>;

    /// Returns a value if available, otherwise `None`.
    ///
    /// By default, this is just a shorthand for `Self::flex_value(...).ok()`.
    fn maybe_flex_value<A>(lua: &'lua Lua, value: Value<'lua>, args: A) -> Option<Self>
    where
        Self: Sized,
        A: IntoLuaMulti<'lua>,
    {
        Self::flex_value(lua, value, args).ok()
    }
}

macro_rules! impl_flex {
    ($lt:lifetime, $ty:ty, $variant:ident ( $var:ident ) => $conv:expr, $expect:literal) => {
        impl<$lt> LuaFlexValue<$lt> for $ty {
            fn flex_value<A>(_lua: &$lt Lua, value: Value<$lt>, args: A) -> Result<Self>
            where
                Self: Sized,
                A: IntoLuaMulti<$lt>,
            {
                match value {
                    Value::$variant($var) => $conv,
                    Value::Function(f) => f.call(args),
                    _ => Err(LuaError::FromLuaConversionError {
                        from: value.type_name(),
                        to: stringify!($ty),
                        message: Some(concat!("expected ", $expect, " or function returning a ", $expect).into()),
                    }),
                }
            }
        }
    };
    ($ty:ty, $variant:ident ( $var:ident ) => $conv:expr, $expect:literal) => {
        impl_flex!('lua, $ty, $variant ( $var ) => $conv, $expect);
    };
}

impl_flex!(bool, Boolean(b) => Ok(b), "bool");
impl_flex!(String, String(s) => Ok(s.to_string_lossy().to_string()), "string");
impl_flex!(i64, Integer(i) => Ok(i), "integer");
impl_flex!(f64, Number(n) => Ok(n), "integer");
impl_flex!('lua, Table<'lua>, Table(t) => Ok(t), "table");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_get_string_fn() {
        let lua = Lua::new();
        let value: Value = lua
            .load("return function(s) return tostring(s) end")
            .call(())
            .unwrap();
        let result: String = String::flex_value(&lua, value, 42).unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn value_get_string_str() {
        let lua = Lua::new();
        let value: Value = lua.load("return 'foo'").call(()).unwrap();
        let result = String::flex_value(&lua, value, ()).unwrap();
        assert_eq!(result, "foo");
    }
}
