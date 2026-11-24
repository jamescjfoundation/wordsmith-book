// SPDX-License-Identifier: MIT OR Apache-2.0
//! The Book project folder: reader, writer, limits and packing; the source (a linked Manuscript project or an owned source) and the offers Book writes beside it.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub use wordsmith_book_model::SectionId;

/// The versions of Manuscript's book view this build reads (plan section 5.4).
pub const BOOK_VIEW_VERSIONS: &[u32] = &[1];

/// Whether this build reads book view `version`.
pub fn reads_book_view(version: u32) -> bool {
    BOOK_VIEW_VERSIONS.contains(&version)
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
