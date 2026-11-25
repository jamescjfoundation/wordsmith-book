// SPDX-License-Identifier: MIT OR Apache-2.0
//! Interface languages (ACC-3) through the core's catalogs (C4): Book's own texts from
//! `assets/i18n/<language>.toml`, the shell's from the shell's catalogs. A text a catalog lacks
//! is shown in English. The language is a preference; by default it follows the computer's.

/// The languages Book's interface comes in: code and name in that language.
pub const LANGUAGES: [(&str, &str); 2] = [("en", "English"), ("es", "Español")];

/// Book's catalogs: language code and catalog text (none for English).
pub const CATALOGS: [(&str, &str); 1] = [("es", include_str!("../../../assets/i18n/es.toml"))];

/// Show the interface in `code` ("" follows the computer). Gives the language now used.
pub fn set(code: &str) -> String {
    jcj_shell::i18n::set_language(code, &CATALOGS)
}
