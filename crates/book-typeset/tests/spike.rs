// SPDX-License-Identifier: MIT OR Apache-2.0
//! The print spike (plan M0.7, decision 0003): one public-domain chapter typeset in a designed
//! layout, its PDF read back letter by letter, built twice to the same bytes, its page images at
//! the trim's size, and its page map.

use std::path::Path;

use wordsmith_book_design::fonts::Role;
use wordsmith_book_typeset::{Block, FirstLine, Look, PrintBook, Section, Span, fonts, typeset};

fn spans(text: &str) -> Vec<Span> {
    // `_underscores_` mark italic in the fixture text.
    let mut out = Vec::new();
    for (i, part) in text.split('_').enumerate() {
        if !part.is_empty() {
            out.push(Span { text: part.to_owned(), italic: i % 2 == 1, ..Span::default() });
        }
    }
    out
}

/// Waverley's first chapter: its paragraphs from the fixture text.
fn chapter() -> Section {
    let text = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels/waverley.txt")).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.iter().position(|l| *l == "Volume I.").unwrap();
    let ch1 = start + lines[start..].iter().position(|l| *l == "CHAPTER I.").unwrap();
    let ch2 = ch1 + 1 + lines[ch1 + 1..].iter().position(|l| *l == "CHAPTER II.").unwrap();
    let mut paras: Vec<String> = Vec::new();
    let mut cur: Vec<&str> = Vec::new();
    for l in &lines[ch1 + 2..ch2] {
        if l.trim().is_empty() {
            if !cur.is_empty() {
                paras.push(cur.join(" "));
                cur.clear();
            }
        } else {
            cur.push(l.trim());
        }
    }
    if !cur.is_empty() {
        paras.push(cur.join(" "));
    }
    // Leave out the note references ([1]) the spike does not set.
    let blocks = paras.iter().map(|p| Block::Para(spans(&p.replace("[1]", "").replace("[2]", "")))).collect();
    Section { id: "chapter-1".into(), number: Some("Chapter One".into()), title: "Introductory".into(), blocks }
}

fn book() -> PrintBook {
    PrintBook { title: "Waverley".into(), author: "Walter Scott".into(), lang: "en".into(), sections: vec![chapter()] }
}

fn look() -> Look {
    let serif = fonts::family(Role::ClassicSerif).unwrap();
    Look { body_font: serif.clone(), heading_font: serif, ornament: "\u{2767}".into(), first_line: FirstLine::SmallCaps, ..Look::default() }
}

fn letters(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

fn source_letters(b: &PrintBook) -> String {
    let mut s = String::new();
    for sec in &b.sections {
        s.push_str(&letters(sec.number.as_deref().unwrap_or("")));
        s.push_str(&letters(&sec.title));
        for bl in &sec.blocks {
            if let Block::Para(spans) = bl {
                for sp in spans {
                    s.push_str(&letters(&sp.text));
                }
            }
        }
    }
    s
}

#[test]
fn a_chapter_prints_every_letter_of_its_source() {
    let b = book();
    let t = typeset(&b, &look(), "2026-11-24").unwrap();
    let pdf = t.pdf().unwrap();
    let pages = jcj_output::read_pages(&pdf, &jcj_output::ReadLimits::default()).unwrap();
    assert_eq!(pages.len(), t.page_count());
    let (_, h) = t.page_size(0).unwrap();

    let top = 0.75 * 72.0;
    let mut got = String::new();
    for p in &pages {
        // The body area only: running heads and folios are not the author's words.
        for g in p.glyphs.iter().filter(|g| {
            // PDF coordinates: y grows upwards from the foot of the page.
            let from_top = f64::from(h) - g.y;
            from_top > top - 2.0 && from_top < f64::from(h) - 0.75 * 72.0 + 2.0
        }) {
            if let Some(text) = &g.text {
                got.push_str(text);
            }
        }
    }
    let want = source_letters(&b);
    let got = letters(&got).replace('\u{2767}', "");
    if got != want {
        let at = got.chars().zip(want.chars()).position(|(a, b)| a != b).unwrap_or(got.len().min(want.len()));
        let around = |s: &str| s.chars().skip(at.saturating_sub(30)).take(60).collect::<String>();
        panic!("letters differ at {at} of {}: printed `{}`, source `{}`", want.len(), around(&got), around(&want));
    }
}

#[test]
fn two_builds_give_the_same_bytes() {
    let a = typeset(&book(), &look(), "2026-11-24").unwrap().pdf().unwrap();
    let b = typeset(&book(), &look(), "2026-11-24").unwrap().pdf().unwrap();
    assert!(a.len() > 10_000);
    assert!(a == b, "the PDF changed between two builds");
}

#[test]
fn a_page_image_has_the_trims_pixel_size_and_the_map_finds_the_chapter() {
    let t = typeset(&book(), &look(), "2026-11-24").unwrap();
    let (w, h, png) = t.page_png(0, 72.0).unwrap();
    assert_eq!((w, h), (432, 648), "6 × 9 in at 72 pixels an inch");
    assert_eq!(&png[..4], b"\x89PNG");
    let (w2, h2, _) = t.page_png(1, 150.0).unwrap();
    assert_eq!((w2, h2), (900, 1350));
    assert_eq!(t.page_map(), vec![wordsmith_book_typeset::Start { id: "chapter-1".into(), page: 1 }]);
    assert!(t.page_count() >= 4, "{} pages", t.page_count());
}

/// Writes the first pages as PNG files under `target/` for a person to look at
/// (`cargo test -p wordsmith-book-typeset --test spike -- --ignored`).
#[test]
#[ignore = "writes files for a person to look at"]
fn write_the_first_pages() {
    let t = typeset(&book(), &look(), "2026-11-24").unwrap();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/spike");
    std::fs::create_dir_all(&dir).unwrap();
    for i in 0..3.min(t.page_count()) {
        let (_, _, png) = t.page_png(i, 110.0).unwrap();
        std::fs::write(dir.join(format!("page-{}.png", i + 1)), png).unwrap();
    }
    std::fs::write(dir.join("chapter.pdf"), t.pdf().unwrap()).unwrap();
}
