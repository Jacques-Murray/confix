use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "A CLI tool to manage application secrets and configurations"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Loads configuration and runs a command
    Run {
        /// Configuration file(s) to load.
        /// Tries to auto-detect .env, .json, or .toml.
        /// Values in later files override earlier ones.
        #[arg(short, long, value_name = "FILE")]
        config: Vec<PathBuf>,

        /// The command to execute
        #[arg(required = true, trailing_var_arg = true)]
        command: Vec<String>,
    },
    // You can add stubs for future features:
    // Encrypt {
    //     #[arg(short, long, value_name = "FILE")]
    //     file: PathBuf,
    // },
    // Decrypt {
    //     #[arg(short, long, value_name = "FILE")]
    //     file: PathBuf,
    // },
}
