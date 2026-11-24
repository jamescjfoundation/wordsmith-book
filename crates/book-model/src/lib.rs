// SPDX-License-Identifier: MIT OR Apache-2.0
//! The Book layer: editions, design choices, overrides and fixes keyed by ids, metadata additions and cleanup decisions. No files, no interface.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

/// The stable id of a section of the book: the id Manuscript gives each item, which Book keys its
/// own layer by (plan section 6.1).
pub use wordsmith_manuscript_model::ItemId as SectionId;
