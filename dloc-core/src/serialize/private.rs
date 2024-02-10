//! Private module containing types needed for serialization and de-serialization.

use std::{
    error::Error,
    fmt::Display,
    fs::File,
    io::{BufReader, BufWriter},
};

use serde::{de::DeserializeOwned, Serialize};

use crate::error::DResult;

/// In order for a game to be usable in group extractor and importer it should
/// impl this trait.
pub trait InternalGroupExtractor: InternalDataSerializer + InternalPlainTextDataSerializer {
    fn internal_new(reader: BufReader<File>) -> DResult<Self>;
    fn internal_write(&self, writer: BufWriter<File>) -> DResult<()>;
}

/// A trait that provides information about the language and error types
/// used for serialization. This allows the serialization logic to be generic
/// over different language and error types.
pub trait InternalSerializerBase: Sized {
    type Language: Copy + Display + Ord + Serialize + DeserializeOwned;
    type Error: Error;
}

/// Trait for internal serialization and deserialization of data.
/// Implementors must define the data type and error type.
/// Provides methods for:
/// - Serializing the data to a serializable type
/// - Deserializing the data and updating the implementor
pub trait InternalDataSerializer: InternalSerializerBase {
    type Output: Serialize + DeserializeOwned;

    fn internal_serialize(&self, languages: &[Self::Language]) -> Self::Output;
    fn internal_deserialize_and_update(&mut self, data: Self::Output) -> Result<(), Self::Error>;
}

/// Serializes and Deserialize data to and from a vector of lines and
/// additional deserialize information. Used for human-readable text
/// serialization.
pub trait InternalPlainTextDataSerializer: InternalSerializerBase {
    type DeserializeInfo: Serialize + DeserializeOwned;

    fn internal_serialize_to_lines(
        &self,
        languages: &[Self::Language],
        add_language_names: bool,
    ) -> (Vec<String>, Self::DeserializeInfo);

    fn internal_deserialize_and_update_from_lines(
        &mut self,
        lines: &[String],
        deinfo: Self::DeserializeInfo,
    ) -> Result<(), Self::Error>;
}
