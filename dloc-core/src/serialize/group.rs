use std::{
    collections::BTreeSet,
    fs::{create_dir_all, File},
    io::{BufRead, BufReader, BufWriter},
    marker::PhantomData,
    ops::Range,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    error::{DResult, Error},
    logger::{Logger, ProgressIterator},
    utils::{self, EofReplacor},
};

use super::{private, SerializeType};

#[derive(Serialize, Deserialize)]
struct TxtGroupDeserializeInfo<D, L: Ord> {
    languages: BTreeSet<L>,
    count: usize,
    info: Vec<TxtGroupLocalDeInfo<D>>,
}

#[derive(Serialize, Deserialize)]
struct TxtGroupLocalDeInfo<D> {
    path: PathBuf,
    range: Range<usize>,
    inner_info: D, // so much duplicate data, but I don't care for now
}

/// A group extractor for extracting all files inside Decima games to a format.
pub struct DecimaGroup<GAME, LOGGER>
where
    GAME: private::InternalGroupExtractor,
    LOGGER: Logger,
{
    base_path: PathBuf,
    files: Vec<PathBuf>,
    logger: LOGGER,
    _phantom: PhantomData<GAME>,
}

impl<GAME, LOGGER> DecimaGroup<GAME, LOGGER>
where
    GAME: private::InternalGroupExtractor,
    LOGGER: Logger,
{
    /// Creates a new DecimaGroup instance by scanning the given input path
    /// for all files with 'core' extention. Returns a DecimaGroup containing
    /// the base path and list of core files found.
    ///
    /// ## Arguments:
    /// * `input`: input dir to load core files from.
    pub fn new<P: AsRef<Path>>(input: P, mut logger: LOGGER) -> DResult<Self> {
        let input = input.as_ref();

        logger.info("Generating file list from input folder.");
        let files: Vec<PathBuf> = utils::generate_file_list(input, Some(&["core"]), usize::MAX)
            .into_iter()
            .map(|p| p.strip_prefix(input).map(Path::to_path_buf))
            .collect::<Result<_, _>>()?;
        logger.good("File list generated.");

        if files.is_empty() {
            return Err(Error::NoFileFound("core"));
        }

        if files.len() == 1 {
            logger.warn("There is just one core file in input folder, it better to use single serialize mode.");
        }

        Ok(Self {
            base_path: input.to_owned(),
            files,
            logger,
            _phantom: PhantomData,
        })
    }

    /// Exports the group data to the given output path in the specified serialization format.
    /// Supported formats are JSON, YAML (if enabled) and plaintext. The plaintext format includes both the
    /// extracted text data and metadata for deserializing (that will be saved next to txt).
    ///
    /// ## Arguments:
    /// * `output`: path to output file.
    /// * `languages`: list of languages to export.
    /// * [`serialize_type`](SerializeType): serialize the output to what type.
    ///
    /// ## Return:
    /// Returns a [`DResult`] indicating whether serialization was successful or not.
    pub fn export<P: AsRef<Path>, L: AsRef<[GAME::Language]>>(
        &mut self,
        output: P,
        languages: L,
        serialize_type: SerializeType,
    ) -> DResult<()> {
        let output = output.as_ref();
        let languages = languages.as_ref();

        match serialize_type {
            SerializeType::Json => {
                self.logger.info("Target serialize format: Json.");
                let locals = self.serialize_locals(languages)?;
                let writer = BufWriter::new(File::create(output)?);
                serde_json::to_writer_pretty(writer, &locals)?;
            }
            #[cfg(feature = "serialize_yaml")]
            SerializeType::Yaml => {
                self.logger.info("Target serialize format: Yaml.");
                let locals = self.serialize_locals(languages)?;
                let writer = BufWriter::new(File::create(output)?);
                serde_yaml::to_writer(writer, &locals)?;
            }
            SerializeType::Txt { add_language_names } => {
                self.logger.info("Target serialize format: Txt.");
                let mut lines = Vec::with_capacity(self.files.len());
                let mut info = Vec::with_capacity(self.files.len());
                let mut count = 0;

                for path in self
                    .files
                    .iter()
                    .progress(&mut self.logger, "Exporting lines from core files")
                {
                    let reader = BufReader::new(File::open(self.base_path.join(path))?);
                    let local = match GAME::internal_new(reader) {
                        Ok(r) => r,
                        Err(e) => match e {
                            Error::NoLocalResource => continue,
                            e => return Err(e),
                        },
                    };
                    let (ilines, deinfo) =
                        local.internal_serialize_to_lines(languages, add_language_names);

                    let len = ilines.len();
                    lines.extend(ilines);
                    info.push(TxtGroupLocalDeInfo {
                        path: path.to_owned(),
                        range: count..count + len,
                        inner_info: deinfo,
                    });
                    count += len;
                }

                let lines: Vec<_> = lines.into_iter().map(EofReplacor::replace_eol).collect();
                self.logger.info("Writing lines to output file.");
                std::fs::write(output, lines.join("\n"))?;
                self.logger.good("Write finished.");
                let deinfo = TxtGroupDeserializeInfo {
                    languages: BTreeSet::from_iter(languages.iter().copied()),
                    count,
                    info,
                };

                self.logger
                    .info("Writing deserialize data to a file next to output.");
                let path = output.with_extension(super::DEINFO_EXT);
                let writer = BufWriter::new(File::create(path)?);
                serde_json::to_writer(writer, &deinfo)?;
                self.logger.good("Write finished.");
            }
        }
        self.logger.good("Serialization finished.");

        Ok(())
    }

    /// Imports previously exported group data from the given input path and deserialize
    /// them into game files in the output directory.
    /// Supported formats are JSON, YAML (if enabled) and plaintext. The plaintext format requires the metadata file
    /// generated during export to be present for deserialization.
    ///
    /// ## Arguments:
    /// * `input`: the input serialized local file.
    /// * `output_dir`: the output dir to save all new created files.
    /// * [`serialize_type`](SerializeType): serialize the output to what type.
    ///
    /// ## Return:
    /// Returns a [`DResult`] indicating whether deserialization and creating
    /// new core files was successful or not.
    pub fn import<P: AsRef<Path>>(
        &mut self,
        input: P,
        output_dir: P,
        serialize_type: SerializeType,
    ) -> DResult<()> {
        let input = input.as_ref();
        let output_dir = output_dir.as_ref();

        self.logger.info("Opening input file.");
        let reader = BufReader::new(File::open(input)?);
        self.logger.good("Input file opened.");

        match serialize_type {
            SerializeType::Json => {
                self.logger.info("Deserialize from Json");
                let locals = serde_json::from_reader(reader)?;
                self.deserialize_locals(locals, output_dir)?;
            }
            #[cfg(feature = "serialize_yaml")]
            SerializeType::Yaml => {
                self.logger.info("Deserialize from Yaml");
                let locals = serde_yaml::from_reader(reader)?;
                self.deserialize_locals(locals, output_dir)?;
            }
            SerializeType::Txt { .. } => {
                self.logger.info("Deserialize from Txt");
                self.logger.info("Reading lines from input file.");
                let lines = reader
                    .lines()
                    .map(|s| s.map(EofReplacor::replace_eol_back))
                    .collect::<std::io::Result<Vec<String>>>()?;
                self.logger.good("Reading lines finished.");

                self.logger.info("Reading deserialize info.");
                let reader = BufReader::new(File::open(input.with_extension(super::DEINFO_EXT))?);
                let deinfo: TxtGroupDeserializeInfo<GAME::DeserializeInfo, GAME::Language> =
                    serde_json::from_reader(reader)?;
                self.logger.info("Reading deserialize info finished.");

                if lines.len() != deinfo.count {
                    return Err(Error::DeserializeError(format!(
                        "Line number doesn't match, expected {} but got {}",
                        deinfo.count,
                        lines.len()
                    )));
                }

                for info in deinfo.info.into_iter().progress(
                    &mut self.logger,
                    "Importing locals and creating new core files",
                ) {
                    if !self.files.contains(&info.path) {
                        // file not found in input folder
                        continue;
                    }

                    let Some(lines) = lines.get(info.range) else {
                        return Err(Error::DeserializeError(format!("Found invalid index when tried to read strings from input. max index: {}. are you sure you didn't modifed the data?", lines.len())));
                    };

                    let reader = BufReader::new(File::open(self.base_path.join(&info.path))?);
                    let mut game = GAME::internal_new(reader)?;
                    game.internal_deserialize_and_update_from_lines(lines, info.inner_info)
                        .map_err(|e| Error::DeserializeError(e.to_string()))?;

                    let path = output_dir.join(info.path);
                    setup_output(&path)?;
                    let writer = BufWriter::new(File::create(path)?);
                    game.internal_write(writer)?;
                }
            }
        }
        self.logger
            .good("Deserialization and update finished and all new files saved to output folder.");

        Ok(())
    }

    fn serialize_locals(
        &mut self,
        languages: &[GAME::Language],
    ) -> DResult<std::collections::HashMap<PathBuf, GAME::Output>> {
        let mut locals = std::collections::HashMap::with_capacity(self.files.len());
        for path in self
            .files
            .iter()
            .progress(&mut self.logger, "Extracting locals")
        {
            let reader = BufReader::new(File::open(self.base_path.join(path))?);
            let game = match GAME::internal_new(reader) {
                Ok(r) => r,
                Err(e) => match e {
                    Error::NoLocalResource => continue,
                    e => return Err(e),
                },
            };
            locals.insert(path.to_owned(), game.internal_serialize(languages));
        }

        Ok(locals)
    }

    fn deserialize_locals(
        &mut self,
        locals: std::collections::HashMap<PathBuf, GAME::Output>,
        output_dir: &Path,
    ) -> DResult<()> {
        for (path, data) in locals
            .into_iter()
            .progress(&mut self.logger, "Importing and creating new core files")
        {
            if !self.files.contains(&path) {
                // file not found in input folder
                continue;
            }

            let reader = BufReader::new(File::open(self.base_path.join(&path))?);
            let mut game = GAME::internal_new(reader)?;
            game.internal_deserialize_and_update(data)
                .map_err(|e| Error::DeserializeError(e.to_string()))?;

            let path = output_dir.join(path);
            setup_output(&path)?;
            let writer = BufWriter::new(File::create(path)?);
            game.internal_write(writer)?;
        }

        Ok(())
    }
}

fn setup_output(path: &Path) -> std::io::Result<()> {
    if path.is_dir() {
        return Ok(());
    }

    create_dir_all(path.with_file_name(""))
}
