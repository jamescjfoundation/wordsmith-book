// SPDX-License-Identifier: MIT OR Apache-2.0
//! Designs: the design schema, the built-in designs, mixing, design files and their versions;
//! and the fonts Book ships, by role.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod builtin;
pub mod design;
pub mod fonts;
pub mod mix;

pub use design::Design;

/// Whether `role` names one of Book's font roles.
pub fn known_role(role: &str) -> bool {
    fonts::Role::from_key(role).is_some()
}
