// SPDX-License-Identifier: MIT OR Apache-2.0
//! The built-in designs (DSN-2), written by the project from general typographic practice
//! (`docs/SPEC.md` section 4.1 lists each one's choices). They ship in the app as design files.

use crate::design::{self, Design};

/// The built-in design files, by id.
pub const FILES: [(&str, &str); 6] = [
    ("classic", include_str!("../../../assets/designs/classic.toml")),
    ("literary", include_str!("../../../assets/designs/literary.toml")),
    ("romance", include_str!("../../../assets/designs/romance.toml")),
    ("thriller", include_str!("../../../assets/designs/thriller.toml")),
    ("fantasy", include_str!("../../../assets/designs/fantasy.toml")),
    ("non-fiction", include_str!("../../../assets/designs/non-fiction.toml")),
];

/// Every built-in design that reads (a test guards that all do).
pub fn all() -> Vec<Design> {
    FILES.iter().filter_map(|(_, text)| design::read(text, &crate::known_role).ok()).collect()
}

/// The built-in design `id`.
pub fn get(id: &str) -> Option<Design> {
    FILES.iter().find(|(i, _)| *i == id).and_then(|(_, text)| design::read(text, &crate::known_role).ok())
}
