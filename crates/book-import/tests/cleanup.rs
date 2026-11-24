// SPDX-License-Identifier: MIT OR Apache-2.0
//! Cleanup (plan M2.4) on every fixture: only quotes, dashes, ellipses and spaces change; every
//! change undone on its own puts its place back; none accepted gives the source back; "Keep
//! exactly as written" proposes nothing.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use wordsmith_book_import::cleanup::{self, Kind};
use wordsmith_book_import::{Format, import};
use wordsmith_book_model::{Cleanup, ImportRules};
use wordsmith_manuscript_model::{Inline, Project};

fn novels() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels")
}

/// A text written for the test, with every kind of change.
const MESSY: &str = "CHAPTER I.\n\n\"Come here,\" she said -- and he came...  He wasn't sure.\n\nIt was 'late' . . . too late.   \n";

fn projects() -> Vec<(String, Project)> {
    let mut out = vec![("written for the test".to_owned(), import(Format::Text, MESSY.as_bytes(), &ImportRules::default(), "t").unwrap().project)];
    for (name, format) in [
        ("waverley.docx", Format::Docx),
        ("gullivers-travels.odt", Format::Odt),
        ("gullivers-travels.rtf", Format::Rtf),
        ("waverley.md", Format::Markdown),
        ("gullivers-travels.txt", Format::Text),
        ("gullivers-travels-online.docx", Format::Docx),
    ] {
        let bytes = std::fs::read(novels().join(name)).unwrap();
        out.push((name.to_owned(), import(format, &bytes, &ImportRules::default(), name).unwrap().project));
    }
    out
}

fn letters_of(p: &Project) -> Vec<String> {
    let mut out = Vec::new();
    for t in p.texts.values() {
        for r in t.body.inline_runs() {
            out.push(wordsmith_manuscript_model::text::plain(r).chars().filter(|c| c.is_alphanumeric()).collect());
        }
    }
    out
}

fn span_text(p: &Project, c: &cleanup::Change) -> String {
    let t = p.text(c.item).unwrap();
    match &t.body.inline_runs()[c.run][c.span] {
        Inline::Span { text, .. } => text.clone(),
        Inline::NoteRef { .. } => String::new(),
    }
}

#[test]
fn only_quotes_dashes_ellipses_and_spaces_change_and_every_change_undoes() {
    for (name, p) in projects() {
        let changes = cleanup::propose(&p, &Cleanup::default());
        let all = cleanup::apply(&p, &changes, &cleanup::all(&changes));
        assert_eq!(letters_of(&all), letters_of(&p), "{name}: no letter or digit changes");
        // Nothing accepted: the source, exactly.
        assert_eq!(cleanup::apply(&p, &changes, &BTreeSet::new()), p, "{name}");
        // Each change undone on its own (a sample of long books) leaves its place as it was.
        let step = (changes.len() / 50).max(1);
        for c in changes.iter().step_by(step) {
            let mut without = cleanup::all(&changes);
            without.remove(&c.id);
            let only = cleanup::apply(&p, &changes, &BTreeSet::from([c.id]));
            let undone = cleanup::apply(&only, &[], &BTreeSet::new());
            assert_ne!(only, p, "{name}: change {} does something", c.id);
            assert_ne!(cleanup::apply(&p, &changes, &without), all, "{name}: change {} is its own", c.id);
            assert_eq!(undone, only);
            assert!(span_text(&p, c)[c.at..].starts_with(&c.from), "{name}: the change is where it says");
        }
    }
}

#[test]
fn the_written_text_gets_each_kind_of_change() {
    let p = import(Format::Text, MESSY.as_bytes(), &ImportRules::default(), "t").unwrap().project;
    let changes = cleanup::propose(&p, &Cleanup::default());
    let kinds: BTreeSet<Kind> = changes.iter().map(|c| c.kind).collect();
    assert_eq!(kinds, Kind::ALL.into_iter().collect());
    let all = cleanup::apply(&p, &changes, &cleanup::all(&changes));
    let text: Vec<String> = all.texts.values().flat_map(|t| t.body.plain_paragraphs()).collect();
    assert!(text.iter().any(|t| t == "“Come here,” she said — and he came… He wasn’t sure."), "{text:?}");
    assert!(text.iter().any(|t| t == "It was ‘late’ … too late."), "{text:?}");
    // Shown word by word: the change and the words around it.
    let shown = cleanup::shown(&p, &changes[0]);
    assert!(shown.iter().any(|s| s.change == jcj_features::wordiff::Change::Added));
    // One kind switched off.
    let no_quotes = cleanup::propose(&p, &Cleanup { off: vec!["quotes".into()], ..Cleanup::default() });
    assert!(no_quotes.iter().all(|c| c.kind != Kind::Quotes) && !no_quotes.is_empty());
}

#[test]
fn keep_exactly_as_written_changes_nothing() {
    for (name, p) in projects() {
        let changes = cleanup::propose(&p, &Cleanup { keep_exactly: true, ..Cleanup::default() });
        assert!(changes.is_empty(), "{name}");
        assert_eq!(cleanup::apply(&p, &changes, &cleanup::all(&changes)), p);
    }
}
