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
    games::hzd::{self, HZDLocal},
    logger::Logger,
    serialize::SerializeData,
};

use crate::{logger::CliLogger, Game};

use super::{
    shared::{HzdAction, SerializeType},
    utils,
};

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
pub struct Single {
    /// Input core file
    #[arg(value_hint = ValueHint::FilePath, value_parser = utils::is_file)]
    input_core: PathBuf,
    /// Serialize type to use
    #[arg(value_enum, default_value_t = SerializeType::default())]
    serialize_type: SerializeType,
    /// Output file
    output: Option<PathBuf>,
    #[command(subcommand)]
    action: HzdAction,
}

impl Single {
    pub fn command(self, game: Game, mut logger: CliLogger) -> anyhow::Result<()> {
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
                    HzdAction::Export {
                        languages,
                        add_language_names,
                    } => {
                        let output = self.output.unwrap_or_else(|| {
                            self.input_core
                                .with_extension(self.serialize_type.extension())
                        });

                        let languages: Vec<_> = languages
                            .into_iter()
                            .filter_map(|s| match hzd::Language::try_from(s.clone()) {
                                Ok(r) => Some(r),
                                Err(_) => {
                                    logger.warn(format!("Invalid language: {s}"));
                                    None
                                }
                            })
                            .collect();

                        if languages.is_empty() {
                            bail!("Didn't found any valid Language.");
                        }

                        logger.info(format!("Selected languages: {languages:?}"));

                        let serialize_type = self.serialize_type.to_core(Some(add_language_names));

                        logger.info(format!(
                            "Serializing locals into {:?} format.",
                            self.serialize_type
                        ));
                        game.serialize(output, languages, serialize_type)?;
                        logger.good("Serialization finished successfully.")
                    }
                    HzdAction::Import {
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
                        game.deserialize_and_update(
                            exported_file,
                            self.serialize_type.to_core(None),
                        )?;
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
            Game::Ds => unimplemented!(),
        }

        Ok(())
    }
}
