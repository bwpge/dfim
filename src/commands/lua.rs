use anyhow::{bail, Result};
use mlua::Table;
use rustyline::{error::ReadlineError, DefaultEditor};

use crate::{cli::LuaArgs, lua};

pub fn exec(args: LuaArgs) -> Result<()> {
    let lua = lua::create_state()?;
    if let Some(block) = args.block {
        Ok(lua.load(process_chunk(&block).unwrap_or(block)).exec()?)
    } else if let Some(path) = args.file {
        Ok(lua.load(path).exec()?)
    } else {
        return repl();
    }
}

fn repl() -> anyhow::Result<()> {
    let lua = lua::create_state()?;
    let jit: Table = lua.globals().get("jit")?;
    let jit_version: String = jit.get("version")?;
    println!(
        "{jit_version}\n\
        To inspect output, prefix input with `=`, e.g.: `={{foo = 'bar'}}`\n\
        To quit, use \"exit\", \"q(uit)\", Ctrl+C, or Ctrl+D"
    );

    let mut rl = DefaultEditor::new()?;
    loop {
        let line = match rl.readline("> ") {
            Ok(line) => match line.as_str() {
                "exit" | "q" | "quit" => break,
                _ => {
                    rl.add_history_entry(&line)?;
                    line
                }
            },
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(e) => bail!(e),
        };

        // try and inspect or print the result of the input to be a bit more
        // helpful. e.g., if user inputs `foo` they probably want to know what
        // `foo` is. this will lead to some syntax errors on more complicated
        // input, so we can just swallow them and execute the block directly.
        let line = if let Some(chunk) = process_chunk(&line) {
            if lua.load(chunk).exec().is_ok() {
                continue;
            }
            line
        } else if !line.contains("print") && lua.load(format!("print({line})")).exec().is_ok() {
            continue;
        } else {
            line
        };

        match lua.load(line).exec() {
            Ok(_) => (),
            Err(e) => eprintln!("{e}"),
        };
    }

    Ok(())
}

/// Process a chunk with similar syntax to Neovim, where a `=` prefix will inspect the output.
fn process_chunk(s: &str) -> Option<String> {
    s.strip_prefix('=')
        .map(|x| format!("print(dfim.inspect({x}))"))
}
