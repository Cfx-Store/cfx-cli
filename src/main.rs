use clap::{Parser, Subcommand};
use simple_logger::SimpleLogger;

mod archive;
mod commands;

use crate::commands::create::handle_create_command;
use crate::commands::unpack::handle_unpack_command;

pub type CfxResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Create,
    Unpack { name: String },
}

fn main() {
    SimpleLogger::new().init().unwrap();

    let cli = Cli::parse();
    let result = match &cli.command {
        Commands::Create => handle_create_command(),
        Commands::Unpack { name } => handle_unpack_command(name),
    };

    match result {
        Ok(_) => log::info!("Command completed successfully"),
        Err(err) => log::error!("Command failed: {}", err),
    }

    log::info!("Press enter to exit...");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}
