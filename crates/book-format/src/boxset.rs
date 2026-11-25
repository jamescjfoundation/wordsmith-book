// SPDX-License-Identifier: MIT OR Apache-2.0
//! Box sets (STR-11): several books in one. Each book's source is opened as it would be on its
//! own and becomes a part of one book view: the part's title is the book's title, its pages are
//! the book's own front matter (its own contents left out: the set has one main contents), its
//! body and its back matter, in order. Book's elements (the set's half title, title page,
//! copyright page and contents) frame the whole.
//!
//! Every book keeps its own section ids, so Book's settings for its chapters, fixes and word
//! checks work on the set as on the book alone.

use wordsmith_book_model::BookProject;
use wordsmith_manuscript_format::book::{Book, FrontKind, Section};
use wordsmith_manuscript_model::{ItemId, Role};

/// Most books in one set.
pub const MAX_BOOKS: usize = 24;

/// The id of the part that holds book `i` of a set (fixed, so a set builds the same every time).
pub fn part_id(i: usize) -> ItemId {
    ItemId::from_counter(0x0B0C_5E70_0000 + i as u64)
}

/// Whether a section is one of the parts a box set makes for its books.
pub fn is_book_part(id: ItemId, books: usize) -> bool {
    (0..books.min(MAX_BOOKS)).any(|i| part_id(i) == id)
}

/// One book view of the books, each with its title (empty: the book's own), in order; none
/// for no books.
pub fn combine(books: &[(String, Book)]) -> Option<Book> {
    let (_, first) = books.first()?;
    let mut out =
        Book { version: first.version, metadata: first.metadata.clone(), front: Vec::new(), body: Vec::new(), back: Vec::new(), assets: Vec::new() };
    for (i, (title, b)) in books.iter().take(MAX_BOOKS).enumerate() {
        let mut children: Vec<Section> = b.front.iter().filter(|s| s.kind != FrontKind::Contents).cloned().collect();
        children.extend(b.body.iter().cloned());
        children.extend(b.back.iter().cloned());
        let title = if title.trim().is_empty() { b.metadata.info.title.clone() } else { title.trim().to_owned() };
        out.body.push(Section {
            id: part_id(i),
            role: Role::Part,
            kind: FrontKind::Other,
            title,
            number: None,
            content: Vec::new(),
            notes: Vec::new(),
            children,
        });
        for a in &b.assets {
            if !out.assets.iter().any(|x| x.path == a.path) {
                out.assets.push(a.clone());
            }
        }
    }
    Some(out)
}

/// Each book of a project's set: its title and source (none when the project is one book).
pub fn books(project: &BookProject) -> Vec<(String, wordsmith_book_model::Source)> {
    project.box_set.iter().take(MAX_BOOKS).map(|b| (b.title.clone(), b.source.clone())).collect()
}
