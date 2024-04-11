mod json;
mod shared;
mod system;

use anyhow::Result;
use mlua::{Lua, Table};

type ApiRegisterFn = for<'lua> fn(&'lua Lua, &'lua Table<'lua>) -> Result<()>;

static REGISTER_FNS: [ApiRegisterFn; 3] = [shared::register, system::register, json::register];

pub(crate) fn create_native_api<'lua>(lua: &'lua Lua, root: &'lua Table<'lua>) -> Result<()> {
    for func in REGISTER_FNS {
        func(&lua, &root)?;
    }

    Ok(())
}
