use thiserror::Error;

pub type DResult<T> = Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    StripPrefix(#[from] std::path::StripPrefixError),

    #[error(transparent)]
    BinRw(#[from] binrw::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[cfg(all(feature = "serialize", feature = "serialize_yaml"))]
    #[error(transparent)]
    SerdeYaml(#[from] serde_yaml::Error),

    #[cfg(feature = "serialize")]
    #[error("Deserialize error: {0}")]
    DeserializeError(String),

    #[error("No \"{0}\" file found")]
    NoFileFound(&'static str),

    #[error("No valid local resource found inside the input")]
    NoLocalResource,
}
