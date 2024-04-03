mod cli;
mod commands;
mod lua;

use std::process::ExitCode;

use clap::Parser;

use crate::cli::Cli;

fn main() -> ExitCode {
    let args = Cli::parse();
    #[cfg(debug_assertions)]
    {
        dbg!(&args);
    }

    if args.version {
        println!("{}", args.version_string());
        return ExitCode::SUCCESS;
    }

    match commands::exec(args) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:?}");
            ExitCode::FAILURE
        }
    }
}
