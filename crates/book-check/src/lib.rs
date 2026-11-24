// SPDX-License-Identifier: MIT OR Apache-2.0
//! Preflight: print and ebook checks on the built files, print profiles and external validators,
//! and the word-for-word check of sources, imports and outputs.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod verify;
