use anyhow::{bail, Result};
use log::trace;
use mlua::{Chunk, Lua, Table};
use rustyline::{error::ReadlineError, history::History, DefaultEditor, Editor};

#[derive(Debug)]
enum Action {
    Execute(String),
    Reset,
    Exit,
}

pub(crate) struct Repl {
    lua: Lua,
}

impl Repl {
    pub(crate) fn new(lua: Lua) -> Result<Self> {
        Ok(Self { lua })
    }

    pub(crate) fn run(self) -> Result<()> {
        trace!("Entering REPL");
        self.print_header()?;
        self.run_impl()
    }

    fn print_header(&self) -> Result<()> {
        let jit: Table = self.lua.globals().get("jit")?;
        let version: String = jit.get("version")?;
        println!(
            "{version}\n\
            To inspect output, prefix input with `=`, e.g.: `={{foo = 'bar'}}`\n\
            To quit, use \"exit\", \"q(uit)\", Ctrl+C, or Ctrl+D"
        );

        Ok(())
    }

    fn run_impl(self) -> Result<()> {
        let mut rl = DefaultEditor::new()?;
        let mut buf = vec![];
        let mut incomplete = false;

        loop {
            let line = match readline(&mut rl, incomplete, buf.is_empty())? {
                Action::Execute(v) => v,
                Action::Reset => {
                    buf.clear();
                    incomplete = false;
                    continue;
                }
                Action::Exit => break,
            };
            buf.push(line);

            match self.load_lines(&buf).exec() {
                Ok(_) => incomplete = false,
                Err(err) => {
                    use mlua::Error::SyntaxError;
                    match err {
                        // most common error will be user providing an identifier,
                        // we can try and be helpful by inspecting it
                        SyntaxError { message, .. }
                            if message.ends_with("'=' expected near '<eof>'") =>
                        {
                            match self.load(inspect_lines(&buf)).exec() {
                                Ok(_) => incomplete = false,
                                Err(e) => {
                                    incomplete = false;
                                    eprintln!("{e}")
                                }
                            }
                        }
                        // traditional lua repl input over multiple lines
                        SyntaxError {
                            incomplete_input: true,
                            ..
                        } => incomplete = true,
                        // other errors
                        _ => {
                            incomplete = false;
                            eprintln!("{err}");
                        }
                    };
                }
            };

            if !incomplete {
                buf.clear();
            }
        }

        Ok(())
    }

    fn load(&self, value: String) -> Chunk {
        self.lua.load(value).set_name("stdin")
    }

    fn load_lines(&self, lines: &[String]) -> Chunk {
        self.lua.load(lines.join(" ")).set_name("stdin")
    }
}

fn readline<H: History>(
    rl: &mut Editor<(), H>,
    incomplete: bool,
    firstline: bool,
) -> Result<Action> {
    let caret = if incomplete { ">> " } else { "> " };
    let action = match rl.readline(caret) {
        Ok(line) => match line.as_str() {
            "exit" | "q" | "quit" => Action::Exit,
            "" => Action::Reset,
            _ => {
                rl.add_history_entry(&line)?;
                if firstline && line.starts_with('=') {
                    Action::Execute(inspect(&line[1..]))
                } else {
                    Action::Execute(line)
                }
            }
        },
        Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => Action::Exit,
        Err(e) => bail!(e),
    };

    Ok(action)
}

fn inspect(value: &str) -> String {
    format!("print(dfim.inspect({}))", value)
}

fn inspect_lines(lines: &[String]) -> String {
    format!("print(dfim.inspect({}))", lines.join(" "))
}
