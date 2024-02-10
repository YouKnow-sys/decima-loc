use std::{fmt::Write, io::Write as _};

use dloc_core::games::hzd;

use crate::logger::CliLogger;

use super::Game;

pub fn print_languages(game: Game, mut logger: CliLogger) -> anyhow::Result<()> {
    match game {
        Game::Hzd => {
            let languages = hzd::Language::ALL_VARIANTS.into_iter().fold(
                String::from("HZD supported languages:\n"),
                |mut s, v| {
                    let _ = writeln!(s, "  - {v}");
                    s
                },
            );
            logger.stdout.write_all(languages.as_bytes())?;
        }
        Game::Ds => unimplemented!(),
    }

    Ok(())
}
