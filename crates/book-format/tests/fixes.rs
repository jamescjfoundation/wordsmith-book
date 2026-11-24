// SPDX-License-Identifier: MIT OR Apache-2.0
//! Light fixes (plan M3.4): offers Manuscript accepts, appended and never rewritten, the
//! Manuscript folder untouched, every decision state, and waiting fixes applied in builds.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use wordsmith_book_format::fixes::{self, State};
use wordsmith_book_format::source;
use wordsmith_book_model::{Fix, Source};
use wordsmith_manuscript_format::offers::{self, Decision};
use wordsmith_manuscript_model::{Block, Inline, Marks, Text};

fn tree(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.insert(p.strip_prefix(dir).unwrap().display().to_string(), std::fs::read(&p).unwrap());
            }
        }
    }
    out
}

/// A copy of the Manuscript fixture and a Book project folder beside it.
fn setup(name: &str) -> (PathBuf, PathBuf) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("fixes").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let ms = dir.join("Gulliver.wsm");
    for (rel, bytes) in tree(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels/gullivers-travels.wsm")) {
        let p = ms.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    let book = dir.join("Book.wsbk");
    std::fs::create_dir_all(&book).unwrap();
    (ms, book)
}

fn fix(id: &str, section: wordsmith_book_model::SectionId, find: &str, replace: &str) -> Fix {
    Fix { id: id.into(), section: Some(section), find: find.into(), replace: replace.into(), note: "typo".into(), made: "2026-11-24".into(), offered: true }
}

#[test]
fn offers_are_read_by_manuscript_appended_and_never_rewritten() {
    let (ms, book) = setup("offers");
    let before = tree(&ms);
    let src = source::open(&book, &Source::Manuscript("../Gulliver.wsm".into())).unwrap();
    let ch = src.book.body[0].children[0].id;
    fixes::offer(&ms, &fix("a", ch, "My father had a small estate", "My father had a little estate")).unwrap();
    let first = std::fs::read(offers::offers_path(&ms)).unwrap();
    let read = offers::read(&first).unwrap();
    assert_eq!(read.version, offers::VERSION);
    assert_eq!(read.offers.len(), 1);
    assert_eq!(read.offers[0].item, ch);
    fixes::offer(&ms, &fix("b", ch, "I was the third of five sons", "I was the third of five sons,")).unwrap();
    let second = offers::read(&std::fs::read(offers::offers_path(&ms)).unwrap()).unwrap();
    assert_eq!(second.offers.len(), 2);
    assert_eq!(second.offers[0], read.offers[0], "the first offer is kept exactly");
    // The same id twice is refused; a newer offers file is never written over.
    assert!(fixes::offer(&ms, &fix("a", ch, "x", "y")).is_err());
    let newer = String::from_utf8(first).unwrap().replace("\"version\": 1", "\"version\": 9");
    std::fs::write(offers::offers_path(&ms), &newer).unwrap();
    assert!(fixes::offer(&ms, &fix("c", ch, "x", "y")).is_err());
    assert_eq!(std::fs::read_to_string(offers::offers_path(&ms)).unwrap(), newer);
    // The project folder itself is untouched: the offers file is beside it.
    assert_eq!(tree(&ms), before);
}

#[test]
fn every_decision_state_is_shown_and_waiting_fixes_apply_in_builds() {
    let (ms, book) = setup("states");
    let src = source::open(&book, &Source::Manuscript("../Gulliver.wsm".into())).unwrap();
    let ch = src.book.body[0].children[0].id;
    let list = vec![
        fix("wait", ch, "My father had a small estate", "My father had a little estate"),
        fix("yes", ch, "I was the third of five sons", "I was the third son of five"),
        fix("no", ch, "He sent me to Emanuel College", "He sent me to a college"),
        fix("gone", ch, "these words are not in the chapter", "anything"),
    ];
    for f in &list {
        fixes::offer(&ms, f).unwrap();
    }
    // The writer, in Manuscript, accepts one (and makes it), rejects one.
    let (mut folder, mut project) = wordsmith_manuscript_format::folder::Folder::open(&ms).unwrap();
    offers::decide(&mut project, "yes", Decision::Accepted).unwrap();
    offers::decide(&mut project, "no", Decision::Rejected).unwrap();
    let t = project.texts.get_mut(&ch).unwrap();
    fixes::apply(&mut t.body, "I was the third of five sons", "I was the third son of five").unwrap();
    folder.save(&project).unwrap();
    let src = source::open(&book, &Source::Manuscript("../Gulliver.wsm".into())).unwrap();
    let states = fixes::states(&list, &src);
    assert_eq!(states[0], State::Waiting);
    assert_eq!(states[1], State::Accepted);
    assert_eq!(states[2], State::Rejected);
    assert!(matches!(&states[3], State::Stale(_)));
    for s in &states {
        assert!(!s.describe().is_empty());
    }
    // A build uses the waiting fix (and not the rejected one).
    let b = fixes::book_for_build(&list, &src);
    let words: String = Text { blocks: fixes::section(&b, ch).unwrap().content.clone(), notes: vec![] }.plain_paragraphs().join(" ");
    assert!(words.contains("My father had a little estate"));
    assert!(words.contains("He sent me to Emanuel College"));
}

#[test]
fn a_fix_keeps_the_marks_and_must_be_found_once() {
    let mut t = Text { blocks: vec![Block::Para(vec![Inline::plain("The "), Inline::marked("old harbour", Marks::italic()), Inline::plain(" lay still.")])], notes: vec![] };
    fixes::apply(&mut t, "old harbour lay", "new harbour lay").unwrap();
    assert_eq!(t.blocks[0], Block::Para(vec![Inline::plain("The "), Inline::marked("new harbour lay", Marks::italic()), Inline::plain(" still.")]));
    let mut twice = Text::from_paragraphs(&["a cat and a cat"]);
    assert!(fixes::apply(&mut twice, "a cat", "a dog").unwrap_err().contains("2 times"));
    assert!(fixes::apply(&mut twice, "missing", "x").is_err());
    assert!(fixes::apply(&mut twice, "a cat and", "two\nlines").is_err());
    // From an edited paragraph: the changed words with enough around them to be found once.
    let section = Text::from_paragraphs(&["the cat sat on the mat", "the cat ran"]);
    assert_eq!(fixes::from_edit("the cat sat on the mat", "the cat sat on the rug", &section), Some(("mat".into(), "rug".into())));
    assert_eq!(fixes::from_edit("the cat ran", "the dog ran", &section), Some(("the cat ran".into(), "the dog ran".into())));
}
