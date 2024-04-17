use anyhow::Result;

use crate::{cli::LuaArgs, config::Config, lua, repl::Repl};

pub fn exec(args: LuaArgs) -> Result<()> {
    let lua = lua::create_state()?;
    Config::load(&lua)?;

    if let Some(block) = args.block {
        lua.load(block).set_name("cli").exec()?;
    } else if let Some(path) = args.file {
        lua.load(path).exec()?;
    } else {
        return Repl::new(lua)?.run();
    }

    Ok(())
}
