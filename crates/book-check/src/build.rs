// SPDX-License-Identifier: MIT OR Apache-2.0
//! Building an edition's print files: the book view (with the fixes that wait on the writer), the
//! assembled book, its design and the edition's profile become a laid-out book.

use std::collections::BTreeMap;

use wordsmith_book_design::mix;
use wordsmith_book_format::assemble::{Assembled, assemble};
use wordsmith_book_model::{BookProject, Edition};
use wordsmith_book_typeset::from_book::{Left, print_book};
use wordsmith_book_typeset::{Look, PrintBook, Typeset, typeset};
use wordsmith_manuscript_format::book::Book;

use crate::profile::{self, Profile};

/// A built print edition.
pub struct Print {
    /// The assembled book.
    pub assembled: Assembled,
    /// What was printed.
    pub book: PrintBook,
    /// How.
    pub look: Look,
    /// The layout.
    pub typeset: Typeset,
    /// The edition's profile.
    pub profile: Profile,
    /// What was left out.
    pub left: Left,
}

/// Build `edition` of the project from `book` (the source's view, with waiting fixes applied),
/// the project's own designs (`designs`, by id), and `images` (a picture's bytes by its path).
pub fn print(project: &BookProject, designs: &BTreeMap<String, String>, book: &Book, edition: &Edition, images: &dyn Fn(&str) -> Option<Vec<u8>>) -> Result<Print, String> {
    if !edition.format.is_print() {
        return Err(format!("the {} is not a printed edition", edition.name.to_lowercase()));
    }
    let profile_id = if edition.profile.is_empty() { "pod-paperback" } else { edition.profile.as_str() };
    let profile = profile::get(profile_id).ok_or_else(|| format!("there is no print profile “{profile_id}”"))?;
    let geometry = profile.geometry(edition)?;
    let design = mix::resolve(&project.design, designs).map_err(|e| e.to_string())?;
    let assembled = assemble(project, book, Some(edition));
    let lang = if book.metadata.info.language.is_empty() { "en".to_owned() } else { book.metadata.info.language.clone() };
    let (pbook, look, left) = print_book(&assembled, &design, &geometry, &lang, images).map_err(|e| e.to_string())?;
    let typeset = typeset(&pbook, &look, if project.date.is_empty() { "2000-01-01" } else { &project.date }).map_err(|e| e.to_string())?;
    Ok(Print { assembled, book: pbook, look, typeset, profile, left })
}
