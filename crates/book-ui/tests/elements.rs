// SPDX-License-Identifier: MIT OR Apache-2.0
//! More front and back matter (plan R2.7, STR-8): each element added through the commands takes
//! its conventional place, and prints and goes into the ebook as its kind asks.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::json;
use wordsmith_book_ui::{BookApp, commands};

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

fn linked(name: &str) -> (BookApp, PathBuf) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("elements").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let ms = dir.join("Gulliver.wsm");
    for (rel, bytes) in tree(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels/gullivers-travels.wsm")) {
        let p = ms.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    let mut app = BookApp::new();
    app.command(commands::NEW, &json!({ "path": dir.join("B.wsbk").display().to_string(), "manuscript": "../Gulliver.wsm" })).unwrap();
    (app, ms)
}

const KINDS: [(&str, &str); 11] = [
    ("foreword", "A word before the voyages begin."),
    ("preface", "Why this edition was made."),
    ("introduction", "The voyages and their times."),
    ("prologue", "The sea was calm the morning he left."),
    ("epilogue", "He never sailed again."),
    ("afterword", "On reading the voyages today."),
    ("newsletter", ""),
    ("excerpt", "The first page of the next book."),
    ("glossary", "Lilliput: an island of very small people\nBrobdingnag — a land of giants"),
    ("endnotes", ""),
    ("questions", "Which voyage changed him most?\nWhy does he return home each time?"),
];

#[test]
fn each_element_is_placed_generated_and_built() {
    let (mut app, _) = linked("all");
    app.command(commands::METADATA_SET, &json!({ "field": "imprint_website", "value": "example.org/books" })).unwrap();
    app.command(commands::EDITION_SET, &json!({ "id": "ebook", "notes": "book-end" })).unwrap();
    assert!(app.command(commands::EDITION_SET, &json!({ "id": "ebook", "notes": "sideways" })).is_err());
    let mut ids = Vec::new();
    // Added in a scrambled order: each goes to its conventional place anyway.
    for (kind, text) in KINDS.iter().rev() {
        let id = app.command(commands::ADD_ELEMENT, &json!({ "kind": kind })).unwrap()["id"].as_str().unwrap().to_owned();
        if !text.is_empty() {
            app.command(commands::SET_ELEMENT, &json!({ "id": id, "text": text })).unwrap();
        }
        ids.push((kind.to_string(), id));
    }
    let b = app.book.as_ref().unwrap();
    let p = &b.files.project;
    for placement in [wordsmith_book_model::Placement::Front, wordsmith_book_model::Placement::Back] {
        let ranks: Vec<u8> = p.elements.iter().filter(|e| e.placement == placement).filter_map(|e| e.kind.map(|k| k.rank(placement))).collect();
        assert!(ranks.windows(2).all(|w| w[0] <= w[1]), "{placement:?}: {ranks:?}");
    }
    // Print.
    let src = b.source.as_ref().unwrap();
    let book = wordsmith_book_format::fixes::book_for_build(&p.fixes, src);
    let mut paperback = p.editions.iter().find(|e| e.format.is_print()).unwrap().clone();
    paperback.notes = wordsmith_book_model::NoteStyle::BookEnd;
    let pr = wordsmith_book_check::build::print(p, &b.files.designs, &book, &paperback, &|_| None).unwrap();
    let map = pr.typeset.page_map();
    for (kind, id) in &ids {
        let piece = format!("element:{id}");
        let i = pr.book.sections.iter().position(|s| s.id == piece).unwrap_or_else(|| panic!("{kind} is not printed"));
        assert!(map.iter().any(|s| s.id == piece), "{kind} has a page");
        let s = &pr.book.sections[i];
        if matches!(kind.as_str(), "prologue" | "epilogue") {
            assert_eq!(s.kind, wordsmith_book_typeset::Kind::Chapter, "{kind} opens like a chapter");
        }
    }
    let first_body = pr.book.sections.iter().position(|s| !s.front).unwrap();
    let pos = |k: &str| pr.book.sections.iter().position(|s| s.id == format!("element:{}", ids.iter().find(|(x, _)| x == k).unwrap().1)).unwrap();
    assert!(pos("foreword") < pos("preface") && pos("preface") < pos("introduction") && pos("introduction") < pos("prologue"));
    assert!(pos("prologue") < first_body + 1 && pos("epilogue") > first_body);
    assert!(pos("epilogue") < pos("afterword") && pos("endnotes") < pos("glossary") && pos("glossary") < pos("questions"));
    let printed: String = pr.typeset.printed_paragraphs().iter().map(|x| x.0.clone()).collect::<Vec<_>>().join("\n");
    assert!(printed.contains("sign up at example.org/books."), "the newsletter page names the website");
    assert!(printed.contains("1. Which voyage") && printed.contains("2. Why does he"), "questions are numbered");
    // The glossary's terms are in small capitals.
    let glossary = &pr.book.sections[pos("glossary")];
    let first = glossary.blocks.iter().find_map(|b| if let wordsmith_book_typeset::Block::Para(sp) = b { Some(sp.clone()) } else { None }).unwrap();
    assert!(first[0].small_caps && first[0].text == "Lilliput");
    // Book-end notes print in the Notes element, not after the book.
    let laid = wordsmith_book_typeset::notes_at_end(&pr.book);
    let host = laid.sections.iter().find(|s| s.holds_notes).unwrap();
    assert!(host.blocks.len() > 3, "the notes are gathered into the Notes element");
    assert!(!laid.sections.iter().any(|s| s.id == "element:notes-at-end"));
    // The ebook: each element's document, with its semantics; notes where the element stands.
    let mut ebook = p.editions.iter().find(|e| !e.format.is_print()).unwrap().clone();
    ebook.notes = wordsmith_book_model::NoteStyle::BookEnd;
    let (_, e) = wordsmith_book_check::build::ebook(p, &b.files.designs, &book, &ebook, &|_| None).unwrap();
    assert!(wordsmith_book_check::epub::check(&e.bytes).is_empty(), "{:#?}", wordsmith_book_check::epub::check(&e.bytes));
    let files = wordsmith_book_check::epub::unzip(&e.bytes).unwrap();
    let all: String = files.iter().filter(|f| f.0.ends_with(".xhtml")).map(|f| String::from_utf8_lossy(&f.2).into_owned()).collect();
    for t in ["foreword", "preface", "introduction", "prologue", "epilogue", "afterword", "glossary"] {
        assert!(all.contains(&format!("epub:type=\"{t}\"")), "{t}");
    }
    let spine: Vec<&str> = e.package.spine.iter().map(|(id, _)| id.as_str()).collect();
    let notes_at = spine.iter().position(|s| *s == "notes").unwrap();
    assert!(notes_at + 1 < spine.len(), "the notes are read where the Notes element stands, not last");
}
