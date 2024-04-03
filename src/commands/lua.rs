use std::fs::read_to_string;

use anyhow::{bail, Context, Result};
use rustyline::{error::ReadlineError, DefaultEditor};

use crate::{cli::LuaArgs, lua};

pub fn exec(args: LuaArgs) -> Result<()> {
    let lua = lua::create_state()?;
    let block = if let Some(block) = args.block {
        block
    } else if let Some(path) = args.file {
        read_to_string(path)?
    } else {
        return repl();
    };

    lua.load(block).exec().context("failed to execute lua")
}

fn repl() -> anyhow::Result<()> {
    let lua = lua::create_state()?;
    let jit: String = lua
        .globals()
        .get::<&str, mlua::Table>("jit")?
        .get("version")?;
    println!("{jit}");
    println!("To quit, use \"exit\", Ctrl+C, or Ctrl+D");
    let mut rl = DefaultEditor::new()?;

    loop {
        let line = match rl.readline("> ") {
            Ok(line) => {
                if line == "exit" {
                    break;
                }
                rl.add_history_entry(&line)?;
                line
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(e) => bail!(e),
        };

        // try and print the result of the input to be a bit more helpful, e.g.,
        // if user inputs `foo` they probably want to know what `foo` is. this
        // will lead to some syntax errors on more complicated input, so we can
        // just swallow them and execute the block directly after.
        if lua.load(format!("print(({line}))")).exec().is_ok() {
            continue;
        };

        match lua.load(line).exec() {
            Ok(_) => (),
            Err(e) => eprintln!("{e}"),
        };
    }

    Ok(())
}
