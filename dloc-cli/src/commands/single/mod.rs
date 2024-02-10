use std::{
    collections::hash_map::DefaultHasher,
    fs::File,
    hash::{Hash, Hasher},
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use anyhow::bail;
use clap::{Parser, ValueHint};
use dloc_core::{
    games::{detect, ds::DSLocal, hzd::HZDLocal},
    logger::Logger,
    serialize::SerializeData,
};

use crate::{logger::CliLogger, Game};

use super::{
    shared::{parse_ds_languages, parse_hzd_languages, Action, SerializeType},
    utils,
};

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
pub struct Single {
    /// Input core file
    #[arg(value_hint = ValueHint::FilePath, value_parser = utils::is_file)]
    input_core: PathBuf,
    /// Output file
    output: Option<PathBuf>,
    #[command(subcommand)]
    action: Action,
}

impl Single {
    pub fn command(
        self,
        game: Game,
        sert: SerializeType,
        mut logger: CliLogger,
    ) -> anyhow::Result<()> {
        let game = match game {
            Game::Auto => {
                let mut reader = BufReader::new(File::open(&self.input_core)?);
                match detect::detect_game(&mut reader)? {
                    detect::GameDetection::Hzd => Game::Hzd,
                    detect::GameDetection::Ds => Game::Ds,
                    detect::GameDetection::Mixed => bail!("Found mixed magic in input core."),
                    detect::GameDetection::Unknown => bail!("Failed to detect any supported game."),
                }
            }
            Game::Hzd => Game::Hzd,
            Game::Ds => Game::Ds,
        };

        logger.info(format!("Selected game: {game:#?}"));
        logger.info(format!("Selected action: {}", self.action.name()));

        logger.info("Opening input core file.");
        let reader = BufReader::new(File::open(&self.input_core)?);
        logger.info("Core file opened.");

        match game {
            Game::Hzd => {
                logger.info("Loading the core file with HZD parser.");
                let mut game = HZDLocal::new(reader)?;
                logger.good("Core file loaded.");

                match self.action {
                    Action::Export {
                        languages,
                        add_language_names,
                    } => {
                        let output = self
                            .output
                            .unwrap_or_else(|| self.input_core.with_extension(sert.extension()));

                        let languages = parse_hzd_languages(languages, &mut logger);

                        if languages.is_empty() {
                            bail!("Didn't found any valid Language.");
                        }

                        logger.info(format!("Selected languages: {languages:?}"));

                        let serialize_type = sert.to_core(Some(add_language_names));

                        logger.info(format!("Serializing locals into {:?} format.", sert));
                        game.serialize(output, languages, serialize_type)?;
                        logger.good("Serialization finished successfully.")
                    }
                    Action::Import {
                        exported_file,
                        dont_skip,
                    } => {
                        let output = self
                            .output
                            .unwrap_or_else(|| self.input_core.with_extension("new.core"));

                        let mut hasher = DefaultHasher::new();
                        game.hash(&mut hasher);
                        let hash_before = hasher.finish();

                        logger.info("Deserializing and updating local files.");
                        game.deserialize_and_update(exported_file, sert.to_core(None))?;
                        logger.good("Deerialization and update finished.");

                        let mut hasher = DefaultHasher::new();
                        game.hash(&mut hasher);
                        let hash_after = hasher.finish();

                        if !dont_skip && hash_before == hash_after {
                            bail!("Nothing changed, write to disk cancelled.");
                        }

                        logger.info("Writing the updated core to output file.");
                        let mut writer = BufWriter::new(File::create(output)?);
                        game.write(&mut writer)?;
                        logger.good("Write finished.");
                    }
                }
            }
            Game::Ds => {
                logger.info("Loading the core file with HZD parser.");
                let mut game = DSLocal::new(reader)?;
                logger.good("Core file loaded.");

                match self.action {
                    Action::Export {
                        languages,
                        add_language_names,
                    } => {
                        let output = self
                            .output
                            .unwrap_or_else(|| self.input_core.with_extension(sert.extension()));

                        let languages = parse_ds_languages(languages, &mut logger);

                        if languages.is_empty() {
                            bail!("Didn't found any valid Language.");
                        }

                        logger.info(format!("Selected languages: {languages:?}"));

                        let serialize_type = sert.to_core(Some(add_language_names));

                        logger.info(format!("Serializing locals into {:?} format.", sert));
                        game.serialize(output, languages, serialize_type)?;
                        logger.good("Serialization finished successfully.")
                    }
                    Action::Import {
                        exported_file,
                        dont_skip,
                    } => {
                        let output = self
                            .output
                            .unwrap_or_else(|| self.input_core.with_extension("new.core"));

                        let mut hasher = DefaultHasher::new();
                        game.hash(&mut hasher);
                        let hash_before = hasher.finish();

                        logger.info("Deserializing and updating local files.");
                        game.deserialize_and_update(exported_file, sert.to_core(None))?;
                        logger.good("Deerialization and update finished.");

                        let mut hasher = DefaultHasher::new();
                        game.hash(&mut hasher);
                        let hash_after = hasher.finish();

                        if !dont_skip && hash_before == hash_after {
                            bail!("Nothing changed, write to disk cancelled.");
                        }

                        logger.info("Writing the updated core to output file.");
                        let mut writer = BufWriter::new(File::create(output)?);
                        game.write(&mut writer)?;
                        logger.good("Write finished.");
                    }
                }
            }
            Game::Auto => unreachable!(),
        }

        Ok(())
    }
}
