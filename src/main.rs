use clap::{Parser, Subcommand};

mod commands;

use crate::commands::create::handle_create_command;

pub type CfxResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Create,
}

fn main() {
    let cli = Cli::parse();
    let _result = match &cli.command {
        Commands::Create => handle_create_command(),
    };
}
