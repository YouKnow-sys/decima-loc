//! Death Stranding

use std::{
    fmt::{Debug, Display},
    io::{Read, Seek, Write},
};

use binrw::{helpers::until_eof, BinWrite, Endian};
use serde::{Deserialize, Serialize};

use crate::{utils::enum_map, DResult, Error};

use structures::Chunk;

use error::DSError;
use structures::ChunkVariants;

mod error;
#[cfg(feature = "serialize")]
mod serialize;
mod structures;

enum_map! {
    /// DS supported languages
    #[derive(Serialize, Deserialize)]
    Language;

    English = 0,
    French = 1,
    Spanish = 2,
    German = 3,
    Italian = 4,
    Dutch = 5,
    Portuguese = 6,
    ChineseTraditional = 7,
    Korean = 8,
    Russian = 9,
    Polish = 10,
    Danish = 11,
    Finnish = 12,
    Norwegian = 13,
    Swedish = 14,
    Japanese = 15,
    LATAMSP = 16,
    LATAMPOR = 17,
    Turkish = 18,
    Arabic = 19,
    ChineseSimplified = 20,
    EnglishUk = 21,
    Greek = 22,
    Czech = 23,
    Hungarian = 24,
}

impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

#[derive(Debug, Clone)]
pub struct LocalResource {
    /// Original index of resource
    pub index: usize,
    pub strings: FixedMap<String>,
}

#[derive(Debug, Hash)]
pub struct DSLocal {
    chunks: Vec<Chunk>,
}

impl DSLocal {
    /// Creates a new [DSLocal] by reading chunks from the reader
    /// until EOF is reached.
    ///
    /// ## Arguments:
    /// * `reader` - The reader to load chunks from. Must implement Read + Seek.
    ///
    /// ## Returns:
    /// A [`DResult`] with a [DSLocal] containing the loaded chunks in case of success.
    /// and a [`Error`] on case of failure.
    pub fn new<R: Read + Seek>(mut reader: R) -> DResult<Self> {
        let chunks: Vec<Chunk> = until_eof(&mut reader, Endian::Little, ())?;

        if !chunks
            .iter()
            .any(|c| matches!(c.variant, ChunkVariants::Localized(_)))
        {
            return Err(Error::NoLocalResource);
        }

        Ok(Self { chunks })
    }

    // Get all Local resources inside the file.
    // So much clone, maybe refactor later.
    pub fn get_locals(&self) -> Vec<LocalResource> {
        self.chunks
            .iter()
            .enumerate()
            .filter_map(|(indx, c)| match &c.variant {
                ChunkVariants::Localized(loc) => Some(LocalResource {
                    index: indx,
                    strings: loc.string_groups.clone().map_inner(|s| s.text.into()),
                }),
                ChunkVariants::Others { .. } => None,
            })
            .collect()
    }

    /// Updates the local resources in this [`DSLocal`] with the provided locals.
    ///
    /// ## Arguments:
    /// * `locals` - The local resources to update this [`DSLocal`] with.
    ///
    /// ## Returns:
    /// Result with [`DSError`] on failure.
    pub fn update_locals(&mut self, locals: Vec<LocalResource>) -> Result<(), DSError> {
        for local in locals {
            let Some(chunk) = self.chunks.get_mut(local.index) else {
                return Err(DSError::InvalidLocalResourceIdx {
                    max: self.chunks.len(),
                    got: local.index,
                });
            };

            match &mut chunk.variant {
                ChunkVariants::Localized(oloc) => {
                    for (lang, str) in local.strings {
                        oloc.string_groups[lang].text = str.into();
                    }
                }
                ChunkVariants::Others { .. } => {
                    return Err(DSError::ResourceNotMatchAtIdx {
                        input: "Localized",
                        original: "Others",
                    });
                }
            }
        }

        Ok(())
    }

    /// Writes the chunks in this [`DSLocal`] to the given writer.
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
