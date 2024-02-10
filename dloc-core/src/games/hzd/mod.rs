//! Horizon Zero Dawn

use std::{
    fmt::{Debug, Display},
    io::{Read, Seek, Write},
};

use binrw::{helpers::until_eof, BinWrite, Endian};
use serde::{Deserialize, Serialize};

use crate::{
    error::{DResult, Error},
    utils::{enum_map, types::U8String},
};

use error::HZDError;
use structures::{Chunk, ChunkVariants, CutsceneStringGroup};

mod error;
#[cfg(feature = "serialize")]
mod serialize;
mod structures;

enum_map! {
    /// HZD availible languages
    #[derive(Serialize, Deserialize)]
    Language;

    English = 0,
    French = 1,
    Spanish = 2,
    German = 3,
    Italian = 4,
    Dutch = 5,
    Portuguese = 6,
    TraditionalChinese = 7,
    Korean = 8,
    Russian = 9,
    Polish = 10,
    Danish = 11,
    Finnish = 12,
    Norwegian = 13,
    Swedish = 14,
    Japanese = 15,
    LatinAmericanSpanish = 16,
    Brazilianportuguese = 17,
    Turkish = 18,
    Arabic = 19,
    SimplifiedChinese = 20,
}

impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

/// LocalResource represents a resource loaded from the game files.
#[derive(Debug, Clone)]
pub struct LocalResource {
    /// Original index of resource
    pub index: usize,
    pub variant: LocalVariants,
}

/// LocalVariants is an enum representing the two variants of
/// localized resources in Horizon Zero Dawn - either a localized
/// string map, or a cutscene dialog map containing vectors of
/// localized strings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LocalVariants {
    Localized(FixedMap<String>),
    Cutscene(FixedMap<Vec<String>>),
}

impl LocalVariants {
    const fn name(&self) -> &'static str {
        match self {
            LocalVariants::Localized(_) => "Localized",
            LocalVariants::Cutscene(_) => "Cutscene",
        }
    }
}

/// HZDLocal is used to load localization resources from Horizon Zero Dawn
/// and store them.
#[derive(Debug, Hash)]
pub struct HZDLocal {
    chunks: Vec<Chunk>,
}

impl HZDLocal {
    /// Creates a new [HZDLocal] by reading chunks from the reader
    /// until EOF is reached.
    ///
    /// ## Arguments:
    /// * `reader` - The reader to load chunks from. Must implement Read + Seek.
    ///
    /// ## Returns:
    /// A [`DResult`] with a HZDLocal containing the loaded chunks in case of success.
    /// and a [`Error`] on case of failure.
    pub fn new<R: Read + Seek>(mut reader: R) -> DResult<Self> {
        let chunks: Vec<Chunk> = until_eof(&mut reader, Endian::Little, ())?;

        if !chunks.iter().any(|c| {
            matches!(
                c.variant,
                ChunkVariants::Cutscene(_) | ChunkVariants::Localized(_)
            )
        }) {
            return Err(Error::NoLocalResource);
        }

        Ok(Self { chunks })
    }

    /// Get all Local resources inside the file.
    // So much clone, maybe refactor later.
    pub fn get_locals(&self) -> Vec<LocalResource> {
        self.chunks
            .iter()
            .enumerate()
            .filter_map(|(indx, c)| match &c.variant {
                ChunkVariants::Localized(loc) => Some(LocalResource {
                    index: indx,
                    variant: LocalVariants::Localized(loc.strings.clone().into()),
                }),
                ChunkVariants::Cutscene(cut) => Some(LocalResource {
                    index: indx,
                    variant: LocalVariants::Cutscene(cut.list.clone().into()),
                }),
                ChunkVariants::Others { .. } => None,
            })
            .collect()
    }

    /// Updates the local resources in this [`HZDLocal`] with the provided locals.
    ///
    /// ## Arguments:
    /// * `locals` - The local resources to update this HZDLocal with.
    ///
    /// ## Returns:
    /// Result with [`HZDError`] on failure.
    pub fn update_locals(&mut self, locals: Vec<LocalResource>) -> Result<(), HZDError> {
        for local in locals {
            let Some(chunk) = self.chunks.get_mut(local.index) else {
                return Err(HZDError::InvalidLocalResourceIdx {
                    max: self.chunks.len(),
                    got: local.index,
                });
            };

            match (local.variant, &mut chunk.variant) {
                (LocalVariants::Localized(loc), ChunkVariants::Localized(oloc)) => {
                    for (lang, str) in loc.into_iter() {
                        oloc.strings[lang] = str.into();
                    }
                }
                (LocalVariants::Cutscene(cut), ChunkVariants::Cutscene(oloc)) => {
                    for (lang, list) in cut.into_iter() {
                        if list.len() != oloc.list[lang].strings_data.len() {
                            return Err(HZDError::CutsceneLinesDoesntMatch {
                                lang,
                                expected: oloc.list[lang].strings_data.len(),
                                got: list.len(),
                            });
                        }

                        for (csd, str) in oloc.list[lang].strings_data.iter_mut().zip(list) {
                            csd.string = str.into();
                        }
                    }
                }
                (l, c) => {
                    return Err(HZDError::ResourceNotMatchAtIdx {
                        input: l.name(),
                        original: c.name(),
                    })
                }
            }
        }

        Ok(())
    }

    /// Writes the chunks in this [`HZDLocal`] to the given writer.
    ///
    /// ## Arguments:
    /// * `writer` - The writer to write the chunks to.
    ///
    /// ## Returns:
    /// Return [`Error`] on failure.
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> DResult<()> {
        self.chunks.write(writer)?;
        Ok(())
    }
}

impl From<FixedMap<U8String>> for FixedMap<String> {
    fn from(value: FixedMap<U8String>) -> Self {
        value.map_inner(String::from)
    }
}

impl From<FixedMap<CutsceneStringGroup>> for FixedMap<Vec<String>> {
    fn from(value: FixedMap<CutsceneStringGroup>) -> Self {
        value.map_inner(|c| {
            c.strings_data
                .into_iter()
                .map(|s| String::from(s.string))
                .collect()
        })
    }
}
