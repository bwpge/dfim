use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

static NAME: &str = env!("CARGO_BIN_NAME");

static AFTER_HELP: &str = "Use -h for short descriptions and --help for more details";

#[derive(Parser, Debug)]
#[command(
    name = NAME,
    author,
    bin_name = NAME,
    version,
    about,
    after_help = AFTER_HELP,
    arg_required_else_help = true,
    disable_help_flag = true,
    disable_version_flag = true,
    help_template = "{bin} {version}\n{author-with-newline}{about-section}\n{all-args}{after-help}",
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    /// Override the configuration file path
    #[arg(long = "config", value_name = "PATH", global = true)]
    pub config_path: Option<PathBuf>,
    /// Suppress all output
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,
    /// Use verbose output (specify multiple for more)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,
    /// Show help information
    #[arg(short, long, action = clap::ArgAction::Help, global = true)]
    pub help: Option<bool>,
    /// Show version information
    #[arg(short = 'V', long)]
    pub version: bool,
}

impl Cli {
    pub fn version_string(&self) -> String {
        let mut value = vec![];

        let version = env!("CARGO_PKG_VERSION");
        if let Some(sha) = option_env!("DFIM_GIT_SHA_SHORT") {
            value.push(format!(
                "{NAME} {version} ({sha} {})",
                env!("VERGEN_GIT_COMMIT_DATE")
            ));
        } else {
            value.push(format!("{NAME} {version}"))
        }

        if self.verbose > 0 {
            if let Some(sha) = option_env!("DFIM_GIT_SHA") {
                value.push(format!("commit-hash: {sha}"));
                value.push(format!("commit-date: {}", env!("VERGEN_GIT_COMMIT_DATE")));
            }
            value.push(format!(
                "build-target: {}",
                env!("VERGEN_CARGO_TARGET_TRIPLE")
            ));
            let profile = if env!("VERGEN_CARGO_DEBUG") == "true" {
                "debug"
            } else {
                "release"
            };
            value.push(format!(
                "build-type: {profile} (opt={})",
                env!("VERGEN_CARGO_OPT_LEVEL")
            ));
        }

        value.join("\n")
    }
}

#[derive(Debug, Subcommand, Clone)]
pub enum Commands {
    /// Execute lua by block, by file, or in a basic REPL
    Lua(LuaArgs),
    /// Show version information
    #[command(hide = true)]
    Version,
}

#[derive(Debug, Clone, Args)]
pub struct LuaArgs {
    /// Execute a block of lua code
    pub block: Option<String>,
    /// Execute a lua file
    #[arg(short, long, conflicts_with = "block")]
    pub file: Option<PathBuf>,
}
