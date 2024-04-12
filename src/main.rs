mod cli;
mod commands;
mod lua;
mod repl;

use std::process::ExitCode;

use clap::Parser;
use fern::colors::{Color, ColoredLevelConfig};
use log::{debug, LevelFilter};

use crate::cli::Cli;

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:?}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> anyhow::Result<()> {
    setup_logger()?;

    let args = Cli::parse();
    #[cfg(debug_assertions)]
    {
        debug!("Handling command:\n{args:#?}");
    }

    if args.version {
        println!("{}", args.version_string());
        return Ok(());
    }

    commands::exec(args)
}

fn setup_logger() -> anyhow::Result<()> {
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .debug(Color::Cyan)
        .trace(Color::BrightBlack);

    let level = if cfg!(debug_assertions) {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    };

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color}{date} {level:<5} {target_color}[{target}]{color} {message}{reset}",
                color = format_args!("\x1b[{}m", colors.get_color(&record.level()).to_fg_str()),
                date = humantime::format_rfc3339_seconds(std::time::SystemTime::now()),
                level = record.level(),
                target_color = format_args!("\x1b[{}m", Color::Yellow.to_fg_str()),
                target = record.target(),
                reset = "\x1b[0m",
            ))
        })
        .level(LevelFilter::Off)
        .level_for("LUA", level)
        .level_for("dfim", level)
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}
