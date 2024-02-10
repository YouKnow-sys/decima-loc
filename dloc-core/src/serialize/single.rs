use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter},
    path::Path,
};

use crate::{
    error::{DResult, Error},
    utils::EofReplacor,
};

use super::{private, SerializeType};

/// Serializes and deserializes data for types that implement the
/// SerializeData trait. This allows serialization to various formats
/// like JSON, YAML, plain text etc. depending on the SerializeType
/// specified.
pub trait SerializeData<T>: Sized
where
    T: private::InternalDataSerializer + private::InternalPlainTextDataSerializer,
{
    /// Serializes the data to the given output path in the specified
    /// serialization format. `languages` specifies the language to include.
    ///
    /// ## Arguments:
    /// * `output`: The path to serialize the data to.
    /// * `languages`: The languages to serialize.
    /// * [`serialize_type`](SerializeType): The serialization format.
    ///
    /// ## Return:
    /// Returns a [`DResult`] indicating whether serialization was successful or not.
    fn serialize<L: AsRef<[T::Language]>, P: AsRef<Path>>(
        &self,
        output: P,
        languages: L,
        serialize_type: SerializeType,
    ) -> DResult<()>;

    /// Deserializes data from the given input path and updates self
    /// with the deserialized data. `serialize_type` specifies the
    /// serialization format of the input data.
    ///
    /// ## Arguments:
    /// * `input`: path to input file to deserialize from.
    /// * [`serialize_type`](SerializeType): The serialization format.
    ///
    /// ## Return:
    /// Returns a [`DResult`] indicating whether deserialization and update
    /// was successful.
    fn deserialize_and_update<P: AsRef<Path>>(
        &mut self,
        input: P,
        serialize_type: SerializeType,
    ) -> DResult<()>;
}

impl<T> SerializeData<T> for T
where
    T: private::InternalDataSerializer + private::InternalPlainTextDataSerializer,
{
    fn serialize<L: AsRef<[<T>::Language]>, P: AsRef<Path>>(
        &self,
        output: P,
        languages: L,
        serialize_type: SerializeType,
    ) -> DResult<()> {
        let output = output.as_ref();
        match serialize_type {
            SerializeType::Json => {
                let value = self.internal_serialize(languages.as_ref());
                let writer = BufWriter::new(File::create(output)?);
                serde_json::to_writer_pretty(writer, &value)?;
            }
            #[cfg(feature = "serialize_yaml")]
            SerializeType::Yaml => {
                let value = self.internal_serialize(languages.as_ref());
                let writer = BufWriter::new(File::create(output)?);
                serde_yaml::to_writer(writer, &value)?;
            }
            SerializeType::Txt { add_language_names } => {
                let (lines, deinfo) =
                    self.internal_serialize_to_lines(languages.as_ref(), add_language_names);
                let lines: Vec<_> = lines.into_iter().map(EofReplacor::replace_eol).collect();

                std::fs::write(output, lines.join("\n"))?;
                let writer =
                    BufWriter::new(File::create(output.with_extension(super::DEINFO_EXT))?);
                serde_json::to_writer(writer, &deinfo)?;
            }
        }

        Ok(())
    }

    fn deserialize_and_update<P: AsRef<Path>>(
        &mut self,
        input: P,
        serialize_type: SerializeType,
    ) -> DResult<()> {
        let input = input.as_ref();

        let reader = BufReader::new(File::open(input)?);
        let data = match serialize_type {
            SerializeType::Json => serde_json::from_reader(reader)?,
            #[cfg(feature = "serialize_yaml")]
            SerializeType::Yaml => serde_yaml::from_reader(reader)?,
            SerializeType::Txt { .. } => {
                let lines = reader
                    .lines()
                    .map(|s| s.map(EofReplacor::replace_eol_back))
                    .collect::<std::io::Result<Vec<String>>>()?;

                let reader = BufReader::new(File::open(input.with_extension(super::DEINFO_EXT))?);

                let deinfo = serde_json::from_reader(reader)?;

                self.internal_deserialize_and_update_from_lines(&lines, deinfo)
                    .map_err(|e| Error::DeserializeError(e.to_string()))?;

                return Ok(());
            }
        };

        self.internal_deserialize_and_update(data)
            .map_err(|e| Error::DeserializeError(e.to_string()))?;

        Ok(())
    }
}
