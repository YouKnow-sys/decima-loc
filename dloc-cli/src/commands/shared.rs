use std::path::PathBuf;

use clap::{Subcommand, ValueEnum, ValueHint};
use dloc_core::serialize::SerializeType as CoreSerializeType;

use super::utils;

#[derive(Clone, Copy, Debug, Default, ValueEnum, PartialEq, Eq)]
pub enum SerializeType {
    #[default]
    Json,
    Yaml,
    Txt,
}

impl SerializeType {
    pub fn to_core(self, add_language_names: Option<bool>) -> CoreSerializeType {
        match self {
            Self::Json => CoreSerializeType::Json,
            Self::Yaml => CoreSerializeType::Yaml,
            Self::Txt => CoreSerializeType::Txt {
                add_language_names: add_language_names.unwrap_or_default(),
            },
        }
    }

    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Txt => "txt",
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum HzdAction {
    /// Export locals from input
    #[command(arg_required_else_help = true)]
    Export {
        /// Languages to export
        #[arg(num_args = 1)]
        languages: Vec<String>,
        /// This option is only used when serialize-type is Txt
        #[arg(short, long)]
        add_language_names: bool,
    },
    /// Import locals back to core and create a new file
    #[command(arg_required_else_help = true)]
    Import {
        /// Exported local file
        #[arg(value_hint = ValueHint::FilePath, value_parser = utils::is_file)]
        exported_file: PathBuf,
        /// Don't skip writing the core to disk if nothing changed, this option don't have any effect in group mode
        #[arg(short, long)]
        dont_skip: bool,
    },
}

impl HzdAction {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Export { .. } => "Export",
            Self::Import { .. } => "Import",
        }
    }
}