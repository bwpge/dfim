use anyhow::Result;

use crate::cli::Cli;

pub fn exec(args: &Cli) -> Result<()> {
    println!("{}", args.version_string());
    Ok(())
}
