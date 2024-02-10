use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub use fixed_map::*;

mod fixed_map;
pub mod types;

/// Generate a file list from input `path`
///
/// # Inputs
/// * `path`: Path to input folder
/// * `ext`: file extension to filter files with, `None` mean no filter
/// * `depth`: depth of the search, normally you should pass [`usize::MAX`] here
/// # Return
/// This function will return a `Vec` of `PathBuf`
#[cfg(feature = "serialize")]
pub(crate) fn generate_file_list(
    path: impl AsRef<Path>,
    extension: Option<&[&str]>,
    depth: usize,
) -> Vec<PathBuf> {
    walkdir::WalkDir::new(path)
        .max_depth(depth)
        .into_iter()
        .filter_map(|f| {
            let f = f.ok()?;
            if f.path().is_dir() {
                return None;
            }
            let Some(ext) = extension else {
                return Some(f.path().to_path_buf()); // No filter
            };
            let file_ext = f.path().extension().and_then(OsStr::to_str)?;
            if ext.contains(&file_ext) {
                return Some(f.path().to_path_buf());
            }
            None
        })
        .collect()
}

#[cfg(feature = "serialize")]
pub trait EofReplacor {
    fn replace_eol(self) -> Self;
    fn replace_eol_back(self) -> Self;
}

// this impl may seem a little more complicated then its need to
// the reason is rust don't have anyway to replace string in place
// by doing it in this way we can save a few allocations.
#[cfg(feature = "serialize")]
impl EofReplacor for String {
    fn replace_eol(self) -> Self {
        if !self.contains(['\r', '\n']) {
            return self;
        }

        let mut buf = String::with_capacity(self.len());
        let mut chars = self.chars();

        while let Some(ch) = chars.next() {
            match ch {
                '\r' => {
                    let br = match chars.as_str().strip_prefix('\n') {
                        Some(s) => {
                            chars = s.chars();
                            "<cf>"
                        }
                        _ => "<cr>",
                    };
                    buf.push_str(br);
                }
                '\n' => buf.push_str("<lf>"),
                ch => buf.push(ch),
            }
        }

        buf
    }

    fn replace_eol_back(self) -> Self {
        if !(self.contains("<cf>") || self.contains("<lf>") || self.contains("<cr>")) {
            return self;
        }

        let mut buf = String::with_capacity(self.len());
        let mut chars = self.chars();

        while let Some(ch) = chars.next() {
            if ch == '<' {
                let mut is_code = true;
                match chars.as_str().get(0..3) {
                    Some("cf>") => buf.push_str("\r\n"),
                    Some("lf>") => buf.push('\n'),
                    Some("cr>") => buf.push('\r'),
                    _ => is_code = false,
                }

                if is_code {
                    chars = chars.as_str()[3..].chars();
                    continue;
                }
            }

            buf.push(ch);
        }

        buf
    }
}

#[cfg(all(feature = "serialize", test))]
mod test {
    use super::*;

    #[test]
    fn replace_eof() {
        assert_eq!(
            String::from("Hi\nHow are you?\rIm Good\r\nGood to know").replace_eol(),
            "Hi<lf>How are you?<cr>Im Good<cf>Good to know",
        );
        assert_eq!(
            String::from("Hi<\nHow are you?\r>Im Good<\r\nGood to know").replace_eol(),
            "Hi<<lf>How are you?<cr>>Im Good<<cf>Good to know",
        );
        assert_eq!(String::from("This<hf>Test").replace_eol(), "This<hf>Test");
    }

    #[test]
    fn replace_eol_back() {
        assert_eq!(
            String::from("Hi<lf>How are you?<cr>Im Good<cf>Good to know").replace_eol_back(),
            "Hi\nHow are you?\rIm Good\r\nGood to know",
        );
        assert_eq!(
            String::from("Hi<<lf>How are you?<cr>>Im Good<<cf>Good to know").replace_eol_back(),
            "Hi<\nHow are you?\r>Im Good<\r\nGood to know",
        );
        assert_eq!(
            String::from("This<hf>Test").replace_eol_back(),
            "This<hf>Test",
        );
    }
}
