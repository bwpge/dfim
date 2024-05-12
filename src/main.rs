mod cli;
mod commands;
mod config;
mod lua;
#[macro_use]
mod macros;
mod repl;
mod source;

use std::{
    io::{stderr, IsTerminal},
    process::ExitCode,
};

use clap::{ColorChoice, Parser};
use fern::{
    colors::{Color, ColoredLevelConfig},
    Dispatch,
};
use log::LevelFilter;

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
    let args = Cli::parse();
    setup_logger(&args)?;
    if let Some(path) = args.config_path.as_ref() {
        config::Config::set_override(path)?;
    }

    #[cfg(debug_assertions)]
    {
        log::debug!("Handling command:\n{args:#?}");
    }

    if args.version {
        println!("{}", args.version_string());
        return Ok(());
    }

    commands::exec(args)
}

fn setup_logger(args: &Cli) -> anyhow::Result<()> {
    let use_color = match args.color.unwrap_or(ColorChoice::Auto) {
        ColorChoice::Auto => stderr().is_terminal(),
        ColorChoice::Always => true,
        ColorChoice::Never => false,
    };

    let level = if let Some(level) = args.log_level {
        level.to_level_filter()
    } else if cfg!(debug_assertions) {
        LevelFilter::Trace
    } else {
        LevelFilter::Warn
    };

    if use_color {
        color_logger()
    } else {
        plain_logger()
    }
    .level(LevelFilter::Off)
    .level_for(env!("CARGO_PKG_NAME"), level)
    .level_for("LUA", level)
    .chain(std::io::stderr())
    .apply()?;

    Ok(())
}

fn color_logger() -> Dispatch {
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .debug(Color::Cyan)
        .trace(Color::BrightBlack);

    Dispatch::new().format(move |out, message, record| {
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
}

fn plain_logger() -> Dispatch {
    Dispatch::new().format(move |out, message, record| {
        out.finish(format_args!(
            "{date} {level:<5} [{target}] {message}",
            date = humantime::format_rfc3339_seconds(std::time::SystemTime::now()),
            level = record.level(),
            target = record.target(),
        ))
    })
}
