use clap::Parser;
use std::process;

mod cli;
mod config;
mod error;
mod runner;

use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    // Run the main logic and store the result
    let result = match cli.command {
        Commands::Run { config, command } => runner::run_command(&config, &command), // Handle future commands here
                                                                                     // Commands::Encrypt { .. } => {
                                                                                     //     println!("Encryption feature not yet implemented.");
                                                                                     //     Ok(0)
                                                                                     // }
    };

    // Handle the result of the program
    match result {
        Ok(exit_code) => {
            // If the spawned command returned a non-zero exit code,
            // we should exit with that code.
            if exit_code != 0 {
                process::exit(exit_code);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
