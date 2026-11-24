// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared helpers for print tests: the test books, assembled, and the PDF's body text.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use wordsmith_book_design::{Design, builtin};
use wordsmith_book_format::assemble::{Assembled, assemble};
use wordsmith_book_model::{BookProject, Source, matter::default_elements};
use wordsmith_book_typeset::from_book::{Geometry, print_book};
use wordsmith_book_typeset::{Look, PrintBook, Typeset, typeset};
use wordsmith_manuscript_format::book::{self, Book};

pub fn novels() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels")
}

/// The Manuscript fixture's book view.
pub fn gulliver() -> Book {
    let (_, p) = wordsmith_manuscript_format::folder::Folder::open(&novels().join("gullivers-travels.wsm")).unwrap();
    book::from_project(&p)
}

/// Waverley, imported from its DOCX, as a book view.
pub fn waverley() -> Book {
    let bytes = std::fs::read(novels().join("waverley.docx")).unwrap();
    let i = wordsmith_book_import::import(wordsmith_book_import::Format::Docx, &bytes, &Default::default(), "Waverley").unwrap();
    let mut p = i.project;
    p.settings.book.author = "Walter Scott".into();
    book::from_project(&p)
}

pub fn project() -> BookProject {
    let mut p = BookProject::new(Source::default(), "2026-11-24");
    p.elements = default_elements();
    p
}

pub fn assembled(b: &Book) -> Assembled {
    let p = project();
    assemble(&p, b, p.edition("paperback"))
}

pub fn print(b: &Book, d: &Design) -> (PrintBook, Look, Typeset) {
    let a = assembled(b);
    let (pb, look, _) = print_book(&a, d, &Geometry::default(), "en", &|_| None).unwrap_or_else(|e| panic!("{}: {e}", d.id));
    let t = typeset(&pb, &look, "2026-11-24").unwrap();
    (pb, look, t)
}

pub fn classic() -> Design {
    builtin::get("classic").unwrap()
}

/// Letters and digits, lower case.
pub fn letters(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

/// The text the PDF prints in each page's body area (heads and folios left out), in order.
pub fn pdf_body_text(t: &Typeset, pdf: &[u8]) -> String {
    let pages =
        jcj_output::read_pages(pdf, &jcj_output::ReadLimits { max_input_bytes: 256 << 20, max_pages: 10_000, max_glyphs_per_page: 20_000 }).unwrap();
    let boxes = jcj_layout::page_boxes(t.layout());
    assert_eq!(pages.len(), boxes.len());
    let mut out = String::new();
    for (p, b) in pages.iter().zip(&boxes) {
        for g in &p.glyphs {
            let from_top = f64::from(b.page.h) - g.y;
            if from_top >= f64::from(b.body.y) - 3.0
                && from_top <= f64::from(b.body.y + b.body.h) + 3.0
                && let Some(text) = &g.text
            {
                out.push_str(text);
            }
        }
    }
    out
}
