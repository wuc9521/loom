mod cli;
mod exec;
mod manifest;
mod model;
mod runner;
mod state;
mod util;

use clap::Parser;

use crate::{
    cli::{Cli, Commands},
    runner::{list_command, run_command, status_command},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run(args) => run_command(args)?,
        Commands::List(args) => list_command(args)?,
        Commands::Status(args) => status_command(args)?,
    }
    Ok(())
}
