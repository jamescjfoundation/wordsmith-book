// SPDX-License-Identifier: MIT OR Apache-2.0
//! Markdown and plain text (plan M2.3): both fixtures imported with their structure and every
//! word checked; separator choices.

use std::path::{Path, PathBuf};

use wordsmith_book_import::{Format, import};
use wordsmith_book_model::ImportRules;
use wordsmith_manuscript_model::Block;

fn novels() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels")
}

fn made(r: &wordsmith_book_import::Report, what: &str) -> usize {
    r.made.get(what).copied().unwrap_or(0)
}

#[test]
fn markdown_fixtures_import_with_their_structure_and_notes() {
    for (name, parts, chapters, front) in [("waverley.md", 2, 72, 0), ("gullivers-travels.md", 4, 39, 2)] {
        let bytes = std::fs::read(novels().join(name)).unwrap();
        let i = import(Format::Markdown, &bytes, &ImportRules::default(), name).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(i.report.passed(), "{name}: {:?}", i.report.words);
        assert_eq!((made(&i.report, "parts"), made(&i.report, "chapters"), made(&i.report, "front matter")), (parts, chapters, front), "{name}: {:?}", i.report.made);
    }
}

#[test]
fn plain_text_imports_by_chapter_lines_with_its_front_matter() {
    let text = std::fs::read(novels().join("gullivers-travels.txt")).unwrap();
    let i = import(Format::Text, &text, &ImportRules::default(), "Gulliver").unwrap();
    assert!(i.report.passed(), "{:?}", i.report.words);
    assert_eq!(made(&i.report, "parts"), 4, "{:?}", i.report.made);
    assert_eq!(made(&i.report, "chapters"), 39);
    // The publisher's note and the letter come before the first part: front matter, kept.
    assert!(made(&i.report, "front matter") >= 2, "{:?}", i.report.made);
    // `_underscores_` became italic, the underscores gone.
    let book = wordsmith_manuscript_format::book::from_project(&i.project);
    let italic = book.front.iter().flat_map(|s| &s.content).any(|b| matches!(b, Block::Para(c) if c.iter().any(|x| x.marks().italic)));
    assert!(italic);
    // Kept literally when the rule is off: every underscore stays.
    let lit = import(Format::Text, &text, &ImportRules { off: vec!["underscores-italic".into()], ..ImportRules::default() }, "Gulliver").unwrap();
    assert!(lit.report.passed());
    let w = wordsmith_book_import::imported_words(&lit.project, &lit.named).0;
    assert!(w.iter().any(|x| x.contains('_')));
}

#[test]
fn separators_are_scene_breaks_as_the_user_chooses() {
    let text = "CHAPTER I.\n\nThe first scene.\n\n-o-\n\nThe second scene.\n\n* * *\n\nThe third scene.\n";
    let default = import(Format::Text, text.as_bytes(), &ImportRules::default(), "t").unwrap();
    assert_eq!(made(&default.report, "scene breaks"), 1, "only the symbol line is a break by default");
    let chosen = ImportRules { break_marks: vec!["-o-".into()], ..ImportRules::default() };
    let with = import(Format::Text, text.as_bytes(), &chosen, "t").unwrap();
    assert_eq!(made(&with.report, "scene breaks"), 2);
    assert!(with.report.passed());
    // A separator the user marked is not the author's words: it leaves no word behind; the
    // check counts it out on both sides.
    let none = ImportRules { off: vec!["symbol-breaks".into()], ..ImportRules::default() };
    let off = import(Format::Text, text.as_bytes(), &none, "t").unwrap();
    assert_eq!(made(&off.report, "scene breaks"), 0);
}
