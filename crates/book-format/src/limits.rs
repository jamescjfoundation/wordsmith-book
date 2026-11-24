// SPDX-License-Identifier: MIT OR Apache-2.0
//! Limits for reading files that are not trusted (plan section 6.1, requirement 5).

/// The largest `book.toml`.
pub const MAX_BOOK_TOML: u64 = 4 << 20;
/// The deepest nesting of tables and arrays in a TOML file.
pub const MAX_DEPTH: usize = 32;
/// The most editions.
pub const MAX_EDITIONS: usize = 100;
/// The most elements of Book's own.
pub const MAX_ELEMENTS: usize = 1000;
/// The most per-section settings.
pub const MAX_SECTIONS: usize = 100_000;
/// The most fixes.
pub const MAX_FIXES: usize = 100_000;
/// The largest design file.
pub const MAX_DESIGN: u64 = 1 << 20;
/// The most design files.
pub const MAX_DESIGNS: usize = 1000;
/// The largest asset (a picture, a font).
pub const MAX_ASSET: u64 = 256 << 20;
/// The most assets.
pub const MAX_ASSETS: usize = 10_000;
/// All assets together.
pub const MAX_ASSETS_TOTAL: u64 = 2 << 30;
/// The largest build record.
pub const MAX_BUILD: u64 = 16 << 20;
/// The most build records.
pub const MAX_BUILDS: usize = 100;
/// The longest file name inside the project.
pub const MAX_NAME: usize = 200;
