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
        InternalDataSerializer, InternalGroupExtractor, InternalSerializerBase,
        InternalTxtDataSerializer,
    },
};

use super::{error::HZDError, structures::ChunkVariants, HZDLocal, Language};

impl InternalGroupExtractor for HZDLocal {
    fn internal_new(reader: BufReader<File>) -> DResult<Self> {
        Self::new(reader)
    }

    fn internal_write(&self, mut writer: BufWriter<File>) -> DResult<()> {
        self.write(&mut writer)
    }
}

impl InternalSerializerBase for HZDLocal {
    type Language = Language;
    type Error = HZDError;
}

// --> serde serializer

#[derive(Serialize, Deserialize)]
pub enum SerdeLocalVariants {
    Localized(BTreeMap<Language, String>),
    Cutscene(BTreeMap<Language, Vec<String>>),
}

impl SerdeLocalVariants {
    const fn name(&self) -> &'static str {
        match self {
            Self::Localized(_) => "Localized",
            Self::Cutscene(_) => "Cutscene",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerdeLocal {
    index: usize,
    #[serde(flatten)]
    variant: SerdeLocalVariants,
}

impl InternalDataSerializer for HZDLocal {
    type Output = Vec<SerdeLocal>;

    fn internal_serialize(&self, languages: &[Self::Language]) -> Self::Output {
        let locals = self.get_locals();
        let mut result = Vec::with_capacity(locals.len());

        macro_rules! add_just_langs {
            ($index:expr, $loc:expr, $variant:ident) => {
                result.push(SerdeLocal {
                    index: $index,
                    variant: SerdeLocalVariants::$variant(
                        $loc.into_iter()
                            .filter(|(l, _)| languages.contains(l))
                            .collect(),
                    ),
                })
            };
        }

        for local in locals {
            match local.variant {
                super::LocalVariants::Localized(loc) => {
                    add_just_langs!(local.index, loc, Localized);
                }
                super::LocalVariants::Cutscene(cut) => {
                    add_just_langs!(local.index, cut, Cutscene);
                }
            }
        }

        result
    }

    fn internal_deserialize_and_update(&mut self, data: Self::Output) -> Result<(), Self::Error> {
        for local in data {
            let Some(chunk) = self.chunks.get_mut(local.index) else {
                return Err(HZDError::InvalidLocalResourceIdx {
                    max: self.chunks.len(),
                    got: local.index,
                });
            };

            match (local.variant, &mut chunk.variant) {
                (SerdeLocalVariants::Localized(loc), ChunkVariants::Localized(oloc)) => {
                    for (lang, str) in loc.into_iter() {
                        oloc.strings[lang] = str.into();
                    }
                }
                (SerdeLocalVariants::Cutscene(cut), ChunkVariants::Cutscene(oloc)) => {
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
}

// --> txt serializer

#[derive(Serialize, Deserialize)]
pub enum TxtLocalVariants {
    Localized,
    Cutscene,
}

impl TxtLocalVariants {
    const fn name(&self) -> &'static str {
        match self {
            Self::Localized => "Localized",
            Self::Cutscene => "Cutscene",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TxtLocalInfo {
    index: usize,
    range: Range<usize>,
    variant: TxtLocalVariants,
}

#[derive(Serialize, Deserialize)]
pub struct TxtDeInfo {
    languages: BTreeSet<Language>,
    add_language_names: bool,
    count: usize,
    info: Vec<TxtLocalInfo>,
}

impl InternalTxtDataSerializer for HZDLocal {
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
            match &chunk.variant {
                ChunkVariants::Localized(loc) => {
                    for lang in languages.iter() {
                        if add_language_names {
                            lines.push(format!("{lang}:: {}", loc.strings[*lang]));
                        } else {
                            lines.push(loc.strings[*lang].to_string());
                        }
                    }

                    info.push(TxtLocalInfo {
                        index,
                        range: count..count + languages.len(),
                        variant: TxtLocalVariants::Localized,
                    });

                    count += languages.len();
                }
                ChunkVariants::Cutscene(cut) => {
                    let mut t_count = 0;
                    for lang in languages.iter() {
                        for str_data in cut.list[*lang].strings_data.iter() {
                            t_count += 1;
                            lines.push(if add_language_names {
                                format!("{lang}:: {}", str_data.string)
                            } else {
                                str_data.string.to_string()
                            })
                        }
                    }

                    info.push(TxtLocalInfo {
                        index,
                        range: count..count + t_count,
                        variant: TxtLocalVariants::Cutscene,
                    });

                    count += t_count;
                }
                ChunkVariants::Others { .. } => (),
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
            return Err(HZDError::LineCountDoesntMatchWithInput {
                expected: deinfo.count,
                got: lines.len(),
            });
        }

        for info in deinfo.info {
            let Some(chunk) = self.chunks.get_mut(info.index) else {
                return Err(HZDError::InvalidLocalResourceIdx {
                    max: self.chunks.len(),
                    got: info.index,
                });
            };

            let Some(lines) = lines.get(info.range) else {
                return Err(HZDError::InvalidIndex {
                    max: lines.len(),
                    invalid_index: info.index,
                });
            };

            match (info.variant, &mut chunk.variant) {
                (TxtLocalVariants::Localized, ChunkVariants::Localized(oloc)) => {
                    for (lang, line) in deinfo.languages.iter().zip(lines) {
                        oloc.strings[*lang] = if deinfo.add_language_names {
                            line.strip_prefix((lang.to_string() + ":: ").as_str())
                                .unwrap_or_else(|| line)
                        } else {
                            line
                        }
                        .to_owned()
                        .into();
                    }
                }
                (TxtLocalVariants::Cutscene, ChunkVariants::Cutscene(oloc)) => {
                    for (lang, lines) in deinfo
                        .languages
                        .iter()
                        .zip(lines.chunks_exact(lines.len() / deinfo.languages.len()))
                    {
                        let str_data = &mut oloc.list[*lang].strings_data;

                        if lines.len() != str_data.len() {
                            return Err(HZDError::CutsceneLinesDoesntMatch {
                                lang: *lang,
                                expected: str_data.len(),
                                got: lines.len() / deinfo.languages.len(),
                            });
                        }

                        for (line, sdata) in lines.iter().zip(str_data) {
                            sdata.string = if deinfo.add_language_names {
                                line.strip_prefix((lang.to_string() + ":: ").as_str())
                                    .unwrap_or_else(|| line)
                            } else {
                                line
                            }
                            .to_owned()
                            .into();
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
}
