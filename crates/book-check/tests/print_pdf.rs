// SPDX-License-Identifier: MIT OR Apache-2.0
//! The print PDF (plan M5.4, PRT-6): for each built-in design on the v0.1 test books, under the
//! `pod-paperback` and `generic-pdf` profiles, the PDF has single pages at trim size, embeds every
//! font, and has no annotations, links or bookmarks; its text, page breaks and page boxes match
//! the golden summaries in `tests/golden/print/` (docs/golden.md). Set `BOOK_BLESS=1` to write
//! the summaries again after a change a person has looked at.

mod common;

use std::fmt::Write as _;
use std::path::PathBuf;

use common::*;
use wordsmith_book_check::build;
use wordsmith_book_design::builtin;
use wordsmith_book_model::Edition;

const LIMITS: jcj_output::ReadLimits = jcj_output::ReadLimits { max_input_bytes: 256 << 20, max_pages: 10_000, max_glyphs_per_page: 20_000 };

fn golden_dir() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/print")
}

/// The words of a page's glyphs, by line from the top.
fn page_lines(page: &jcj_output::PdfPage) -> Vec<String> {
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

fn ends(s: &str, n: usize, first: bool) -> String {
    let words: Vec<&str> = s.split_whitespace().collect();
    if first { words.iter().take(n).copied().collect::<Vec<_>>().join(" ") } else { words[words.len().saturating_sub(n)..].join(" ") }
}

/// A build's summary: the page size, then per page its folio, section, line count and the first
/// and last words of its body.
fn summary(pr: &build::Print, pdf: &[u8]) -> String {
    let pages = jcj_output::read_pages(pdf, &LIMITS).unwrap();
    let t = &pr.typeset;
    let (w, h) = t.page_size(0).unwrap();
    let mut out = format!("pages {}\nsize {:.2} x {:.2} pt\ngutter {:.3} in\n", pages.len(), w, h, t.gutter());
    for (i, page) in pages.iter().enumerate() {
        let lines = page_lines(page);
        let text = lines.join(" ");
        let section = t.section_of_page(i).unwrap_or_default();
        let folio = t.folio(i);
        if text.is_empty() {
            writeln!(out, "{:>4} [{folio}] {section}: blank", i + 1).unwrap();
        } else {
            writeln!(out, "{:>4} [{folio}] {section}: {} lines | {} … {}", i + 1, lines.len(), ends(&text, 4, true), ends(&text, 4, false)).unwrap();
        }
    }
    out
}

/// PRT-6 on one build's PDF.
fn check_pdf(name: &str, pr: &build::Print, pdf: &[u8], trim: (f64, f64)) {
    let facts = jcj_output::read_facts(pdf, &LIMITS).unwrap();
    assert_eq!(facts.pages.len(), pr.typeset.page_count(), "{name}");
    assert!(!facts.fonts.is_empty(), "{name}");
    assert!(facts.unembedded_fonts().is_empty(), "{name}: fonts not embedded: {:?}", facts.unembedded_fonts());
    assert_eq!(facts.annotation_count(), 0, "{name}: annotations");
    assert_eq!(facts.bookmarks, 0, "{name}: bookmarks");
    for (i, p) in facts.pages.iter().enumerate() {
        // Single pages at trim size: no crop marks or slug around the page.
        assert!((p.media.width() - trim.0).abs() < 0.01 && (p.media.height() - trim.1).abs() < 0.01, "{name} page {}: {:?}", i + 1, p.media);
        assert_eq!(p.crop, p.media, "{name} page {}", i + 1);
        assert!(p.trim.is_none_or(|t| t == p.media), "{name} page {}", i + 1);
        assert_eq!(p.rotation, 0);
    }
}

fn build_one(book: &wordsmith_manuscript_format::book::Book, design: &str, profile: &str, trim: &str) -> build::Print {
    let mut p = project();
    p.design.id = design.into();
    let mut e = Edition::paperback();
    e.trim = trim.into();
    e.profile = profile.into();
    build::print(&p, &Default::default(), book, &e, &|_| None).unwrap_or_else(|err| panic!("{design}/{profile}: {err}"))
}

fn golden(book_name: &str, book: &wordsmith_manuscript_format::book::Book, profile: &str, trim: &str) {
    let bless = std::env::var_os("BOOK_BLESS").is_some();
    let prof = wordsmith_book_check::profile::get(profile).unwrap();
    let t = prof.trim(trim).unwrap();
    let size = (f64::from(t.width) * 72.0, f64::from(t.height) * 72.0);
    let mut failures = Vec::new();
    for design in builtin::all() {
        let name = format!("{book_name}-{}-{profile}", design.id);
        let pr = build_one(book, &design.id, profile, trim);
        let pdf = pr.typeset.pdf().unwrap();
        check_pdf(&name, &pr, &pdf, size);
        let got = summary(&pr, &pdf);
        let path = golden_dir().join(format!("{name}.txt"));
        if bless {
            std::fs::create_dir_all(golden_dir()).unwrap();
            std::fs::write(&path, &got).unwrap();
            continue;
        }
        let want = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{name}: no golden summary; run with BOOK_BLESS=1"));
        if got != want {
            let line = got.lines().zip(want.lines()).position(|(a, b)| a != b).unwrap_or(got.lines().count().min(want.lines().count()));
            failures.push(format!(
                "{name}: line {}: got `{}`, golden `{}`",
                line + 1,
                got.lines().nth(line).unwrap_or(""),
                want.lines().nth(line).unwrap_or("")
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn gulliver_in_every_design_matches_its_golden_print_pdf() {
    golden("gulliver", &gulliver(), "pod-paperback", "5.5x8.5");
}

#[test]
fn waverley_in_every_design_matches_its_golden_print_pdf() {
    golden("waverley", &waverley(), "pod-paperback", "6x9");
}

#[test]
fn the_generic_profile_gives_a_print_pdf_with_no_gutter_table() {
    golden("gulliver", &gulliver(), "generic-pdf", "5x8");
}
