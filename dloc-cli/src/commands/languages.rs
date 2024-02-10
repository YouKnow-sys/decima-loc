use std::{fmt::Write, io::Write as _};

use dloc_core::games::{ds, hzd};

use crate::logger::CliLogger;

use super::Game;

pub fn print_languages(game: Game, mut logger: CliLogger) -> anyhow::Result<()> {
    match game {
        Game::Auto => {
            logger.stdout.write_all(
                "Auto can't be used in language command, input the game directly.".as_bytes(),
            )?;
        }
        Game::Hzd => {
            let languages = hzd::Language::ALL_VARIANTS.into_iter().fold(
                String::from("Horizon Zero Dawn supported languages:\n"),
                |mut s, v| {
                    let _ = writeln!(s, "  - {v}");
                    s
                },
            );
            logger.stdout.write_all(languages.as_bytes())?;
        }
        Game::Ds => {
            let languages = ds::Language::ALL_VARIANTS.into_iter().fold(
                String::from("Death Stranding supported languages:\n"),
                |mut s, v| {
                    let _ = writeln!(s, "  - {v}");
                    s
                },
            );
            logger.stdout.write_all(languages.as_bytes())?;
        }
    }

    Ok(())
}
