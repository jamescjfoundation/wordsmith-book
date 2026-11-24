// SPDX-License-Identifier: MIT OR Apache-2.0
//! The word-for-word check (plan M2.1) on every fixture: against itself it passes; against a copy
//! with one word changed it finds that word, with context.

use std::path::{Path, PathBuf};

use wordsmith_book_check::verify::{Check, book_words, compare, document_words, words};

fn novels() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels")
}

/// `w` with word `at` replaced.
fn one_changed(w: &[String], at: usize) -> Vec<String> {
    let mut c = w.to_vec();
    c[at] = format!("{}x", c[at]);
    c
}

fn check_both_ways(name: &str, w: &[String]) {
    assert!(w.len() > 1000, "{name} has its words ({})", w.len());
    assert_eq!(compare(w, w), Ok(w.len()), "{name} against itself");
    let at = w.len() / 2;
    let d = compare(w, &one_changed(w, at)).unwrap_err();
    assert_eq!(d.at, at, "{name}");
    assert_eq!(d.expected.as_ref(), Some(&w[at]));
    assert_eq!(d.before.len(), wordsmith_book_check::verify::CONTEXT.min(at));
    assert!(d.to_string().contains(&format!("word {}", at + 1)));
}

#[test]
fn every_text_fixture_checks_against_itself_and_finds_one_changed_word() {
    for name in ["waverley.txt", "gullivers-travels.txt", "middlemarch.txt"] {
        let text = std::fs::read_to_string(novels().join(name)).unwrap();
        check_both_ways(name, &words(&text));
    }
}

#[test]
fn every_document_fixture_checks_against_itself_and_finds_one_changed_word() {
    for (name, format) in [
        ("waverley.docx", jcj_interchange::Format::Docx),
        ("waverley.odt", jcj_interchange::Format::Odt),
        ("waverley.rtf", jcj_interchange::Format::Rtf),
        ("waverley.md", jcj_interchange::Format::Markdown),
        ("gullivers-travels.docx", jcj_interchange::Format::Docx),
        ("gullivers-travels.odt", jcj_interchange::Format::Odt),
        ("gullivers-travels.rtf", jcj_interchange::Format::Rtf),
        ("gullivers-travels.md", jcj_interchange::Format::Markdown),
        ("gullivers-travels-online.docx", jcj_interchange::Format::Docx),
        ("middlemarch.docx", jcj_interchange::Format::Docx),
    ] {
        let bytes = std::fs::read(novels().join(name)).unwrap();
        let doc = jcj_interchange::import(format, &bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
        check_both_ways(name, &document_words(&doc));
    }
}

#[test]
fn the_manuscript_fixture_checks_against_itself_and_finds_one_changed_word() {
    let (_, project) = wordsmith_manuscript_format::folder::Folder::open(&novels().join("gullivers-travels.wsm")).unwrap();
    let book = wordsmith_manuscript_format::book::from_project(&project);
    let with_notes = book_words(&book, true);
    let without = book_words(&book, false);
    assert!(with_notes.len() > without.len(), "notes add words");
    check_both_ways("the Manuscript fixture", &with_notes);
    assert!(matches!(Check::of(&with_notes, &without), Check::Differs(_)));
}
