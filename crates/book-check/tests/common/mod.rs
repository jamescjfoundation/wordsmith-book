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

/// Bounds for reading the test books' PDFs.
pub const PDF_LIMITS: jcj_output::ReadLimits = jcj_output::ReadLimits { max_input_bytes: 256 << 20, max_pages: 10_000, max_glyphs_per_page: 20_000 };

/// The words of a page's glyphs, by line from the top.
pub fn page_lines(page: &jcj_output::PdfPage) -> Vec<String> {
    let mut lines: Vec<(f64, String)> = Vec::new();
    for g in &page.glyphs {
        let Some(t) = &g.text else { continue };
        match lines.iter_mut().find(|(y, _)| (y - g.y).abs() < 1.0) {
            Some((_, s)) => s.push_str(t),
            None => lines.push((g.y, t.clone())),
        }
    }
    lines.sort_by(|a, b| b.0.total_cmp(&a.0));
    // Contents leaders (runs of dots) are shown as one mark.
    let leaders = |s: &str| {
        let mut out = String::new();
        for c in s.chars() {
            if c == '.' && out.ends_with("..") {
                if !out.ends_with(" … ") {
                    out.truncate(out.len() - 2);
                    out.push_str(" … ");
                }
            } else if !(c == '.' && out.ends_with(" … ")) {
                out.push(c);
            }
        }
        out
    };
    lines.into_iter().map(|(_, s)| leaders(&s).split_whitespace().collect::<Vec<_>>().join(" ")).filter(|s| !s.is_empty()).collect()
}

pub fn ends(s: &str, n: usize, first: bool) -> String {
    let words: Vec<&str> = s.split_whitespace().collect();
    if first { words.iter().take(n).copied().collect::<Vec<_>>().join(" ") } else { words[words.len().saturating_sub(n)..].join(" ") }
}

/// A build's summary: the page size, then per page its folio, section, line count and the first
/// and last words of its body.
pub fn summary(pr: &wordsmith_book_check::build::Print, pdf: &[u8]) -> String {
    let pages = jcj_output::read_pages(pdf, &PDF_LIMITS).unwrap();
    let t = &pr.typeset;
    let (w, h) = t.page_size(0).unwrap();
    let mut out = format!("pages {}\nsize {:.2} x {:.2} pt\ngutter {:.3} in\n", pages.len(), w, h, t.gutter());
    for (i, page) in pages.iter().enumerate() {
        let lines = page_lines(page);
        let text = lines.join(" ");
        let section = t.section_of_page(i).unwrap_or_default();
        let folio = t.folio(i);
        if text.is_empty() {
            out.push_str(&format!("{:>4} [{folio}] {section}: blank\n", i + 1));
        } else {
            out.push_str(&format!(
                "{:>4} [{folio}] {section}: {} lines | {} … {}\n",
                i + 1,
                lines.len(),
                ends(&text, 4, true),
                ends(&text, 4, false)
            ));
        }
    }
    out
}
