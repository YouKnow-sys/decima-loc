use thiserror::Error;

use super::Language;

#[derive(Debug, Error)]
pub enum HZDError {
    #[error("Found Invalid index when tried to update local resource, max resource: {max}, buf found: {got}")]
    InvalidLocalResourceIdx { max: usize, got: usize },
    #[error("Cutscene lines for language {lang} doesn't match with original, expected {expected} got {got}")]
    CutsceneLinesDoesntMatch {
        lang: Language,
        expected: usize,
        got: usize,
    },
    #[error("Resource not match at index, input {input} but original {original}")]
    ResourceNotMatchAtIdx {
        input: &'static str,
        original: &'static str,
    },
    #[error("Input line count doesn't match with with the deserialize info says, expected {expected} but got {got}")]
    LineCountDoesntMatchWithInput { expected: usize, got: usize },
    #[error("Found invalid index when tried to read strings from input. max index: {max}, found in index {invalid_index}. are you sure you didn't modifed the data?")]
    InvalidIndex { max: usize, invalid_index: usize },
}
