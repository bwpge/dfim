mod cli;

use std::process::ExitCode;

use clap::Parser;

use crate::cli::Cli;

fn main() -> ExitCode {
    let args = Cli::parse();
    dbg!(&args);

    if args.version || matches!(args.command, Some(cli::Commands::Version)) {
        println!("{}", args.version_string());
    }

    ExitCode::SUCCESS
}
