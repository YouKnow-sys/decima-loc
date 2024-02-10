use clap::Parser;

use commands::{Commands, Game, LogLevel};
use logger::CliLogger;

mod commands;
mod logger;

#[derive(Debug, Parser)]
#[command(name = "DLOC CLI", author, about = include_str!("../logo.txt"))]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Input game
    #[arg(value_enum, default_value_t = Game::default())]
    game: Game,
    /// Global program log level
    #[arg(short, long, name = "LEVEL", value_enum, global = true, default_value_t = LogLevel::default())]
    log_level: LogLevel,
}

impl Cli {
    fn run(self) -> anyhow::Result<()> {
        let logger = CliLogger::new(self.log_level);
        self.command.command(self.game, logger)
    }
}

fn main() -> anyhow::Result<()> {
    Cli::parse().run()
}
