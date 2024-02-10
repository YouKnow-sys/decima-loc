use clap::{Subcommand, ValueEnum};

use crate::logger::CliLogger;

mod group;
mod languages;
mod shared;
mod single;
mod utils;

#[derive(Clone, Copy, Debug, Default, ValueEnum, PartialEq, Eq)]
pub enum Game {
    /// Try to auto detect the game from input
    #[default]
    Auto,
    /// Horizon zero dawn
    #[value(alias = "HorizonZeroDown")]
    Hzd,
    /// Death stranding
    #[value(alias = "DeathStranding")]
    Ds,
}

/// Different program log levels.
#[derive(Clone, Copy, Debug, Default, ValueEnum, PartialEq, Eq)]
pub enum LogLevel {
    /// Show all log messages and progress
    #[default]
    #[value(alias = "All")]
    A,
    /// Show all log messages but no progress
    #[value(alias = "NoProgress")]
    P,
    /// Only show Error and Warn messages
    #[value(alias = "Warn")]
    W,
    /// Only show Error messages
    #[value(alias = "Error")]
    E,
    /// Show nothing
    #[value(alias = "Nothing")]
    N,
}

#[derive(Debug, Subcommand)]
#[command(arg_required_else_help = true)]
pub enum Commands {
    /// Extract or import strings from a single core file
    Single(single::Single),
    /// Extract or import strings from a group of core files
    Group(group::Group),
    /// See supported languages for each game
    Languages,
}

impl Commands {
    pub fn command(self, game: Game, logger: CliLogger) -> anyhow::Result<()> {
        match self {
            Commands::Single(c) => c.command(game, logger),
            Commands::Group(c) => c.command(game, logger),
            Commands::Languages => languages::print_languages(game, logger),
        }
    }
}
