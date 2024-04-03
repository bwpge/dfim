mod lua;
mod version;

use crate::cli::{Cli, Commands};

pub fn exec(args: Cli) -> anyhow::Result<()> {
    match args.command {
        Some(Commands::Lua(args)) => self::lua::exec(args),
        Some(Commands::Version) => version::exec(&args),
        _ => todo!(),
    }
}
