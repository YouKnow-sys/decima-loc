use std::path::PathBuf;

use anyhow::bail;
use clap::{Parser, ValueHint};
use dloc_core::{
    games::{ds::DSLocal, hzd::HZDLocal},
    logger::Logger,
    serialize::DecimaGroup,
};

use crate::{logger::CliLogger, Game};

use super::{
    shared::{parse_ds_languages, parse_hzd_languages, Action, SerializeType},
    utils,
};

#[derive(Debug, Parser)]
pub struct Group {
    /// Input folder that have all the core files inside it
    #[arg(value_hint = ValueHint::DirPath, value_parser = utils::is_dir)]
    input_dir: PathBuf,
    /// Serialize type to use
    #[arg(value_enum, default_value_t = SerializeType::default())]
    serialize_type: SerializeType,
    /// Output file
    output: Option<PathBuf>,
    #[command(subcommand)]
    action: Action,
}

impl Group {
    pub fn command(self, game: Game, mut logger: CliLogger) -> anyhow::Result<()> {
        logger.info(format!("Selected game: {game:#?}"));
        logger.info(format!("Selected action: {}", self.action.name()));

        match game {
            Game::Hzd => match self.action {
                Action::Export {
                    languages,
                    add_language_names,
                } => {
                    let output = self.output.unwrap_or_else(|| {
                        self.input_dir
                            .with_extension(self.serialize_type.extension())
                    });

                    let languages = parse_hzd_languages(languages, &mut logger);

                    if languages.is_empty() {
                        bail!("Didn't found any valid Language.");
                    }

                    logger.info(format!("Selected languages: {languages:?}"));

                    let serialize_type = self.serialize_type.to_core(Some(add_language_names));

                    logger.info(format!(
                        "Serializing locals into {:?} format.",
                        self.serialize_type
                    ));

                    DecimaGroup::<HZDLocal, _>::new(self.input_dir, logger)?.export(
                        output,
                        languages,
                        serialize_type,
                    )?;
                }
                Action::Import {
                    exported_file,
                    dont_skip: _,
                } => {
                    let output = self
                        .output
                        .unwrap_or_else(|| self.input_dir.with_extension("new"));

                    DecimaGroup::<HZDLocal, _>::new(self.input_dir, logger)?.import(
                        exported_file,
                        output,
                        self.serialize_type.to_core(None),
                    )?;
                }
            },
            Game::Ds => match self.action {
                Action::Export {
                    languages,
                    add_language_names,
                } => {
                    let output = self.output.unwrap_or_else(|| {
                        self.input_dir
                            .with_extension(self.serialize_type.extension())
                    });

                    let languages = parse_ds_languages(languages, &mut logger);

                    if languages.is_empty() {
                        bail!("Didn't found any valid Language.");
                    }

                    logger.info(format!("Selected languages: {languages:?}"));

                    let serialize_type = self.serialize_type.to_core(Some(add_language_names));

                    logger.info(format!(
                        "Serializing locals into {:?} format.",
                        self.serialize_type
                    ));

                    DecimaGroup::<DSLocal, _>::new(self.input_dir, logger)?.export(
                        output,
                        languages,
                        serialize_type,
                    )?;
                }
                Action::Import {
                    exported_file,
                    dont_skip: _,
                } => {
                    let output = self
                        .output
                        .unwrap_or_else(|| self.input_dir.with_extension("new"));

                    DecimaGroup::<DSLocal, _>::new(self.input_dir, logger)?.import(
                        exported_file,
                        output,
                        self.serialize_type.to_core(None),
                    )?;
                }
            },
        }

        Ok(())
    }
}
