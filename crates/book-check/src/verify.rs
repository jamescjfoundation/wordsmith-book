// SPDX-License-Identifier: MIT OR Apache-2.0
//! The word-for-word check (C1, the core's `jcj_features::verify`) on Book's inputs and outputs:
//! the words of a book view, of a document, of an output read back, compared in reading order.

pub use jcj_features::verify::{CONTEXT, Check, Difference, compare};
use wordsmith_manuscript_format::book::{Book, Section};
use wordsmith_manuscript_model::{Role, Text};

/// The soft hyphen, a hint where a word may break (TXT-5): never part of a word.
const SOFT_HYPHEN: char = '\u{AD}';

/// A text's words (the core's split), soft hyphens left out.
pub fn words(text: &str) -> Vec<String> {
    jcj_features::verify::words(&text.replace(SOFT_HYPHEN, ""))
}

/// A document's words (the core's reading), soft hyphens left out.
pub fn document_words(doc: &jcj_doc::Document) -> Vec<String> {
    jcj_features::verify::document_words(doc).into_iter().map(|w| w.replace(SOFT_HYPHEN, "")).filter(|w| !w.is_empty()).collect()
}

fn section_words(s: &Section, notes: bool, out: &mut Vec<String>) {
    if matches!(s.role, Role::Part | Role::Chapter) {
        out.extend(words(&s.title));
    }
    let t = Text { blocks: s.content.clone(), notes: if notes { s.notes.clone() } else { Vec::new() } };
    for p in t.plain_paragraphs() {
        out.extend(words(&p));
    }
    for c in &s.children {
        section_words(c, notes, out);
    }
}

/// The words of a book view in reading order: each part's and chapter's title, then its text;
/// with `notes`, each section's notes after its text.
pub fn book_words(b: &Book, notes: bool) -> Vec<String> {
    let mut out = Vec::new();
    for s in b.front.iter().chain(&b.body).chain(&b.back) {
        section_words(s, notes, &mut out);
    }
    out
}

/// The words of a book view with every note after all the text, in order (an edition whose
/// notes are gathered at the book's end).
pub fn book_words_notes_last(b: &Book) -> Vec<String> {
    fn notes(s: &Section, out: &mut Vec<String>) {
        for n in &s.notes {
            for p in &n.paras {
                out.extend(words(&wordsmith_manuscript_model::text::plain(p)));
            }
        }
        for c in &s.children {
            notes(c, out);
        }
    }
    let mut out = book_words(b, false);
    for s in b.front.iter().chain(&b.body).chain(&b.back) {
        notes(s, &mut out);
    }
    out
}

/// The words of a text's paragraphs (blocks, then notes).
pub fn text_words(t: &Text) -> Vec<String> {
    t.plain_paragraphs().iter().flat_map(|p| words(p)).collect()
}
