//! Serialize and deserialize related trait and functions

pub use group::DecimaGroup;
pub use single::SerializeData;

mod group;
pub(crate) mod private;
mod single;

const DEINFO_EXT: &str = "deinfo.json";

/// An enum representing the different serialization formats supported.
///
/// This includes JSON, plain text and Yaml (if enabled). The enum variants correspond to
/// each of these formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SerializeType {
    Json,
    #[cfg(feature = "serialize_yaml")]
    Yaml,
    Txt {
        add_language_names: bool,
    },
}
