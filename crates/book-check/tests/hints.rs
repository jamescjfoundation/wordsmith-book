// SPDX-License-Identifier: MIT OR Apache-2.0
//! No-break text and hyphenation hints (plan R2.15, TXT-5): honoured in print (a phrase kept
//! together never splits across lines; a hinted word breaks only at its marks), kept in the
//! ebook (no-break spaces and soft hyphens in its text), and the word checks still pass.

mod common;

use common::*;
use wordsmith_book_check::{build, preflight};
use wordsmith_book_model::{Edition, Hints, Severity};
use wordsmith_book_typeset::typeset;
use wordsmith_manuscript_format::book::{Book, Section};
use wordsmith_manuscript_model::{Block, Inline, Role};

/// The places "incomprehensibility" may break, as its hint marks them.
const HINT: &str = "incom-prehen-sibility";

fn first_chapter(sections: &mut [Section]) -> Option<&mut Section> {
    for s in sections {
        if s.role == Role::Chapter {
            return Some(s);
        }
        if let Some(c) = first_chapter(&mut s.children) {
            return Some(c);
        }
    }
    None
}

/// Gulliver's Travels with its first chapter replaced by paragraphs that need the hints.
fn book() -> Book {
    let mut b = gulliver();
    let c = first_chapter(&mut b.body).unwrap();
    c.children.clear();
    c.content = (0..12)
        .map(|i| {
            Block::Para(vec![Inline::plain(format!(
                "{}Then Mr. Smith spoke of incomprehensibility, and Mr. Smith left; the incomprehensibility of it all weighed {} kg on Mr. Smith.",
                "word ".repeat(i),
                10 + i
            ))])
        })
        .collect();
    b
}

fn hinted() -> wordsmith_book_model::BookProject {
    let mut p = project();
    p.design.id = "classic".into();
    p.hints = Hints { keep: vec!["Mr. Smith".into()], hyphenate: vec![HINT.into()] };
    p
}

#[test]
fn print_keeps_phrases_together_and_breaks_words_only_at_their_marks() {
    let p = hinted();
    let source = book();
    let mut pr = build::print(&p, &Default::default(), &source, &Edition::paperback(), &|_| None).unwrap();
    // A narrow measure, so lines break often.
    pr.look.trim = (3.0, 6.0);
    pr.look.margins = (0.5, 0.5, 0.5, 0.5);
    pr.look.gutter.clear();
    pr.typeset = typeset(&pr.book, &pr.look, "2026-11-24").unwrap();
    let lines = pr.typeset.author_lines();
    assert!(lines.len() > 100, "{}", lines.len());
    let mut hyphenated = 0;
    for w in lines.windows(2) {
        let (a, b) = (w[0].0.trim_end(), w[1].0.trim_start());
        assert!(!(a.ends_with("Mr.") && b.starts_with("Smith")), "“Mr. Smith” split: {a:?} / {b:?}");
        if w[0].1 {
            hyphenated += 1;
            let tail: String = a.rsplit(|c: char| c.is_whitespace()).next().unwrap().chars().filter(|c| *c != '\u{AD}').collect();
            assert!(["incom", "incomprehen"].contains(&tail.as_str()), "broken at {tail:?}, not at a mark");
        }
    }
    assert!(hyphenated > 0, "the narrow measure breaks the hinted word somewhere");
    // Every letter printed is the book's, and the words are the source's.
    let pdf = pr.typeset.pdf().unwrap();
    let f = preflight::print_words(&pr, &pdf, &source);
    assert!(f.is_empty(), "{f:#?}");
}

#[test]
fn the_ebook_keeps_the_hints_in_its_text_and_its_words_are_the_sources() {
    let p = hinted();
    let source = book();
    let (_, e) = build::ebook(&p, &Default::default(), &source, &Edition::ebook(), &|_| None).unwrap();
    let text: String = wordsmith_book_check::epub::unzip(&e.bytes)
        .unwrap()
        .into_iter()
        .filter(|f| f.0.ends_with(".xhtml"))
        .map(|f| String::from_utf8(f.2).unwrap())
        .collect();
    assert!(text.contains("Mr.\u{A0}Smith"));
    assert!(!text.contains("Mr. Smith"), "every one kept");
    assert!(text.contains("incom\u{AD}prehen\u{AD}sibility"));
    let f = preflight::ebook_findings(&e.bytes, &source, &e.cover_problems, false, false);
    assert!(!f.iter().any(|x| x.rule == "words" && x.severity == Severity::Blocks), "{f:#?}");
}

#[test]
fn without_hints_the_book_is_set_as_before() {
    let mut p = hinted();
    p.hints = Hints::default();
    let pr = build::print(&p, &Default::default(), &book(), &Edition::paperback(), &|_| None).unwrap();
    assert!(pr.typeset.author_lines().iter().all(|l| !l.0.contains('\u{A0}') && !l.0.contains('\u{AD}')));
    // And the hints' problems are found.
    let bad = Hints { keep: vec![" ".into()], hyphenate: vec!["two words".into(), "-ab".into(), "a!b".into()] };
    assert_eq!(bad.problems().len(), 4, "{:?}", bad.problems());
    assert!(hinted().hints.problems().is_empty());
}
