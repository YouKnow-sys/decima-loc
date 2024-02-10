use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{BufReader, BufWriter},
    ops::Range,
};

use serde::{Deserialize, Serialize};

use crate::{
    error::DResult,
    serialize::private::{
        InternalDataSerializer, InternalGroupExtractor, InternalPlainTextDataSerializer,
        InternalSerializerBase,
    },
};

use super::{error::DSError, structures::ChunkVariants, DSLocal, Language};

impl InternalGroupExtractor for DSLocal {
    fn internal_new(reader: BufReader<File>) -> DResult<Self> {
        Self::new(reader)
    }

    fn internal_write(&self, mut writer: BufWriter<File>) -> DResult<()> {
        self.write(&mut writer)
    }
}

impl InternalSerializerBase for DSLocal {
    type Language = Language;
    type Error = DSError;
}

// --> serde serializer

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalResource {
    /// Original index of resource
    pub index: usize,
    pub strings: BTreeMap<Language, String>,
}

impl InternalDataSerializer for DSLocal {
    type Output = Vec<LocalResource>;

    fn internal_serialize(&self, languages: &[Self::Language]) -> Self::Output {
        self.chunks
            .iter()
            .enumerate()
            .filter_map(|(index, chunk)| match &chunk.variant {
                ChunkVariants::Localized(loc) => Some(LocalResource {
                    index,
                    strings: loc
                        .string_groups
                        .iter()
                        .filter(|(l, _)| languages.contains(l))
                        .map(|(l, s)| (l, s.text.to_string()))
                        .collect(),
                }),
                ChunkVariants::Others { .. } => None,
            })
            .collect()
    }

    fn internal_deserialize_and_update(&mut self, data: Self::Output) -> Result<(), Self::Error> {
        for local in data {
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
}

// --> txt serializer

#[derive(Serialize, Deserialize)]
pub struct TxtLocalInfo {
    index: usize,
    range: Range<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct TxtDeInfo {
    languages: BTreeSet<Language>,
    add_language_names: bool,
    count: usize,
    info: Vec<TxtLocalInfo>,
}

impl InternalPlainTextDataSerializer for DSLocal {
    type DeserializeInfo = TxtDeInfo;

    fn internal_serialize_to_lines(
        &self,
        languages: &[Self::Language],
        add_language_names: bool,
    ) -> (Vec<String>, Self::DeserializeInfo) {
        let languages = BTreeSet::from_iter(languages.iter().copied());

        let mut count = 0;
        let mut lines = Vec::new();
        let mut info = Vec::new();

        for (index, chunk) in self.chunks.iter().enumerate() {
            if let ChunkVariants::Localized(loc) = &chunk.variant {
                for lang in languages.iter() {
                    if add_language_names {
                        lines.push(format!("{lang}:: {}", loc.string_groups[*lang].text));
                    } else {
                        lines.push(loc.string_groups[*lang].text.to_string());
                    }

                    info.push(TxtLocalInfo {
                        index,
                        range: count..count + languages.len(),
                    });

                    count += languages.len();
                }
            }
        }

        let info = TxtDeInfo {
            languages,
            add_language_names,
            count,
            info,
        };

        (lines, info)
    }

    fn internal_deserialize_and_update_from_lines(
        &mut self,
        lines: &[String],
        deinfo: Self::DeserializeInfo,
    ) -> Result<(), Self::Error> {
        if lines.len() != deinfo.count {
            return Err(DSError::LineCountDoesntMatchWithInput {
                expected: deinfo.count,
                got: lines.len(),
            });
        }

        for info in deinfo.info {
            let Some(chunk) = self.chunks.get_mut(info.index) else {
                return Err(DSError::InvalidLocalResourceIdx {
                    max: self.chunks.len(),
                    got: info.index,
                });
            };

            match &mut chunk.variant {
                ChunkVariants::Localized(oloc) => {
                    let Some(lines) = lines.get(info.range) else {
                        return Err(DSError::InvalidIndex {
                            max: lines.len(),
                            invalid_index: info.index,
                        });
                    };

                    assert_eq!(
                        lines.len(), deinfo.languages.len(),
                        "range of lines doesn't match with languages count, did you changed something in deinfo file?"
                    );

                    for (lang, line) in deinfo.languages.iter().zip(lines) {
                        oloc.string_groups[*lang].text = if deinfo.add_language_names {
                            line.strip_prefix((lang.to_string() + "::").as_str())
                                .unwrap_or_else(|| line)
                                .trim_start()
                        } else {
                            line
                        }
                        .to_owned()
                        .into();
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
}
