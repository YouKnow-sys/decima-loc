use std::path::PathBuf;

use anyhow::bail;
use clap::{Parser, ValueHint};
use dloc_core::{
    games::hzd::{self, HZDLocal},
    logger::Logger,
    serialize::DecimaGroup,
};

use crate::{logger::CliLogger, Game};

use super::{
    shared::{HzdAction, SerializeType},
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
    action: HzdAction,
}

impl Group {
    pub fn command(self, game: Game, mut logger: CliLogger) -> anyhow::Result<()> {
        logger.info(format!("Selected game: {game:#?}"));
        logger.info(format!("Selected action: {}", self.action.name()));

        match game {
            Game::Hzd => match self.action {
                HzdAction::Export {
                    languages,
                    add_language_names,
                } => {
                    let output = self.output.unwrap_or_else(|| {
                        self.input_dir
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

                    DecimaGroup::<HZDLocal, _>::new(self.input_dir, logger)?.export(
                        output,
                        languages,
                        serialize_type,
                    )?;
                }
                HzdAction::Import {
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
            Game::Ds => unimplemented!(),
        }

        Ok(())
    }
}
