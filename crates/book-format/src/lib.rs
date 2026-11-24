// SPDX-License-Identifier: MIT OR Apache-2.0
//! The Book project folder (plan section 6, `docs/format.md`): `book.toml` and its schema
//! versions, the user's designs, assets and build records, limits for files that are not
//! trusted, and saving only what changed. The source (a linked Manuscript project or an owned
//! source) and the offers Book writes beside a linked project are read and written through
//! Manuscript's published format crate, never by hand.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod folder;
pub mod limits;
pub mod source;
pub mod toml_file;

pub use folder::{Files, Folder, Saved};
pub use wordsmith_book_model::SectionId;

/// A Book project folder's extension (decision 0002).
pub const EXTENSION: &str = "wsbk";
/// A packed Book project's extension (decision 0002).
pub const PACKED_EXTENSION: &str = "wsbkz";

/// The versions of Manuscript's book view this build reads (plan section 5.4).
pub const BOOK_VIEW_VERSIONS: &[u32] = &[1];

/// Whether this build reads book view `version`.
pub fn reads_book_view(version: u32) -> bool {
    BOOK_VIEW_VERSIONS.contains(&version)
}

/// Why a project could not be read or written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The folder has no `book.toml` with a schema.
    NotABook,
    /// The project was written by a newer version of the app.
    Newer {
        /// The schema the project declares.
        found: u32,
        /// The newest schema this build reads.
        supported: u32,
    },
    /// A file or a count is over a limit.
    TooLarge {
        /// What was too large.
        what: String,
        /// The limit.
        limit: u64,
    },
    /// A file is damaged or not in the format.
    Damaged {
        /// The file, inside the project.
        file: String,
        /// What is wrong.
        why: String,
    },
    /// Reading or writing a file failed.
    Io {
        /// The file.
        file: String,
        /// The system's message.
        why: String,
    },
    /// The source cannot be used.
    Source(String),
    /// Something could not be written.
    Write(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotABook => write!(f, "this is not a Wordsmith Book project (no book.toml with a schema)"),
            Error::Newer { found, supported } => write!(
                f,
                "this project was saved by a newer version of Wordsmith Book (format {found}; this version reads up to {supported}). Update the app to open it."
            ),
            Error::TooLarge { what, limit } => write!(f, "{what} is larger than the limit of {limit}"),
            Error::Damaged { file, why } => write!(f, "{file} is damaged: {why}"),
            Error::Io { file, why } => write!(f, "{file}: {why}"),
            Error::Source(why) => write!(f, "the book's source cannot be used: {why}"),
            Error::Write(why) => write!(f, "could not write the project: {why}"),
        }
    }
}

impl std::error::Error for Error {}

/// Check a source's path: an owned source is a folder name inside the project; a linked one is a
/// relative path to a Manuscript folder or packed project.
pub fn check_source(source: &wordsmith_book_model::Source) -> Result<(), Error> {
    match source {
        wordsmith_book_model::Source::Owned(name) => {
            if !folder::safe_name(name) || name.contains('/') || !name.ends_with(".wsm") {
                return Err(Error::Source(format!("an owned source is a `.wsm` folder inside the project, not `{name}`")));
            }
        }
        wordsmith_book_model::Source::Manuscript(path) => {
            if path.is_empty() || path.chars().any(char::is_control) || path.len() > 4096 {
                return Err(Error::Source("a linked Manuscript project needs a path".into()));
            }
            if !(path.ends_with(".wsm") || path.ends_with(".wsmz") || path.ends_with(".wsm/")) {
                return Err(Error::Source(format!("`{path}` is not a Manuscript project (`.wsm` or `.wsmz`)")));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_book_view_manuscript_gives() {
        assert!(reads_book_view(wordsmith_manuscript_format::book::VERSION));
        assert!(!reads_book_view(2));
    }
}
