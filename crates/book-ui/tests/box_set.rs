// SPDX-License-Identifier: MIT OR Apache-2.0
//! Box sets (plan R3.3, STR-11): two books in one. Each keeps its own front and back matter in
//! its part; the set has one main contents; the print and ebook editions' words are each book's
//! words, book by book, in order; the set is one undo step, saved with the project.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::json;
use wordsmith_book_format::assemble::{What, assemble};
use wordsmith_book_format::boxset;
use wordsmith_book_model::{ElementKind, Severity};
use wordsmith_book_ui::{BookApp, commands};
use wordsmith_manuscript_format::book::FrontKind;

fn tree(dir: &Path, base: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            tree(&p, base, out);
        } else {
            out.insert(p.strip_prefix(base).unwrap().to_owned(), std::fs::read(&p).unwrap());
        }
    }
}

fn novels() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels")
}

/// A book linked to Gulliver's Travels, with Waverley imported beside it as a source of the
/// project's own.
fn set_up(name: &str) -> (BookApp, PathBuf) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let mut files = BTreeMap::new();
    let src = novels().join("gullivers-travels.wsm");
    tree(&src, &src, &mut files);
    for (rel, bytes) in files {
        let p = dir.join("Gulliver.wsm").join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    let mut app = BookApp::new();
    let path = dir.join("Set.wsbk");
    app.command(commands::NEW, &json!({ "path": path.display().to_string(), "manuscript": "../Gulliver.wsm", "date": "2026-11-24" })).unwrap();
    let bytes = std::fs::read(novels().join("waverley.docx")).unwrap();
    let mut i = wordsmith_book_import::import(wordsmith_book_import::Format::Docx, &bytes, &Default::default(), "Waverley").unwrap();
    i.project.settings.book.author = "Walter Scott".into();
    wordsmith_book_format::source::write_owned(&path, "waverley.wsm", &i.project).unwrap();
    (app, path)
}

fn make_set(app: &mut BookApp) -> serde_json::Value {
    app.command(
        commands::BOX_SET,
        &json!({ "books": [
            { "title": "Gulliver's Travels", "manuscript": "../Gulliver.wsm" },
            { "title": "Waverley", "owned": "waverley.wsm" },
        ] }),
    )
    .unwrap()
}

/// Whether `part` appears in `all` as one run, in order.
fn contains_run(all: &[String], part: &[String]) -> bool {
    !part.is_empty() && all.windows(part.len()).any(|w| w == part)
}

#[test]
fn each_book_keeps_its_matter_under_one_main_contents() {
    let (mut app, _) = set_up("box-set-structure");
    let alone = app.book.as_ref().unwrap().source.as_ref().unwrap().book.clone();
    let v = make_set(&mut app);
    let parts = v["parts"].as_array().unwrap();
    assert_eq!(parts.iter().map(|p| p["title"].as_str().unwrap()).collect::<Vec<_>>(), ["Gulliver's Travels", "Waverley"]);
    let b = app.book.as_ref().unwrap();
    let src = b.source.as_ref().unwrap();
    // Gulliver's own pages, its contents apart, open its part; its chapters follow, numbered as
    // in the book alone.
    let first = &src.book.body[0];
    let own_front: Vec<_> = alone.front.iter().filter(|s| s.kind != FrontKind::Contents).map(|s| s.id).collect();
    assert_eq!(first.children.iter().take(own_front.len()).map(|s| s.id).collect::<Vec<_>>(), own_front);
    assert_eq!(first.children.len(), own_front.len() + alone.body.len() + alone.back.len());
    // One main contents (Book's), listing both books.
    let a = assemble(&b.files.project, &src.book, b.files.project.edition("paperback"));
    let contents =
        a.pieces().filter(|p| matches!(p.what, What::Element(ElementKind::Contents)) || matches!(p.what, What::Matter(FrontKind::Contents))).count();
    assert_eq!(contents, 1);
    let part_titles: Vec<&str> = a.pieces().filter(|p| matches!(p.what, What::Part)).map(|p| p.title.as_str()).collect();
    assert!(part_titles.starts_with(&["Gulliver's Travels"]) && part_titles.contains(&"Waverley"), "{part_titles:?}");
    assert!(boxset::is_book_part(first.id, 2));
}

#[test]
fn the_set_s_editions_carry_each_book_s_words_in_order() {
    let (mut app, _) = set_up("box-set-words");
    make_set(&mut app);
    let b = app.book.as_ref().unwrap();
    let src = b.source.as_ref().unwrap();
    let books: Vec<Vec<String>> = src
        .book
        .body
        .iter()
        .map(|part| {
            let one = wordsmith_manuscript_format::book::Book {
                version: src.book.version,
                metadata: src.book.metadata.clone(),
                front: Vec::new(),
                body: part.children.clone(),
                back: Vec::new(),
                assets: Vec::new(),
            };
            wordsmith_book_check::verify::book_words(&one, true)
        })
        .collect();
    assert_eq!(books.len(), 2);
    // The ebook, read back: each book's words as one run, Gulliver's before Waverley's.
    let (_, e) =
        wordsmith_book_check::build::ebook(&b.files.project, &b.files.designs, &src.book, &wordsmith_book_model::Edition::ebook(), &|p| b.picture(p))
            .unwrap();
    let words = wordsmith_book_check::epub::words(&e.bytes, true).unwrap();
    for (i, w) in books.iter().enumerate() {
        assert!(contains_run(&words, w), "book {i}'s words are not one run in the ebook");
    }
    let at = |w: &Vec<String>| words.windows(w.len()).position(|x| x == w.as_slice()).unwrap();
    assert!(at(&books[0]) < at(&books[1]));
    // Preflight finds no word difference in either edition (a set this long may pass the paper's
    // page limit; that is its own finding).
    for edition in ["paperback", "ebook"] {
        let r = wordsmith_book_ui::preflight::run(b, edition, &Default::default()).unwrap();
        let words: Vec<_> = r.of(Severity::Blocks).iter().filter(|f| f.rule == "words").map(|f| f.message.clone()).collect();
        assert!(words.is_empty(), "{edition}: {words:?}");
    }
}

#[test]
fn the_set_is_one_undo_step_and_is_saved() {
    let (mut app, path) = set_up("box-set-undo");
    let alone = app.book.as_ref().unwrap().source.as_ref().unwrap().book.body.len();
    make_set(&mut app);
    assert_eq!(app.book.as_ref().unwrap().source.as_ref().unwrap().book.body.len(), 2);
    // Refused: one book, a book twice, a book that does not open.
    for bad in [
        json!({ "books": [{ "manuscript": "../Gulliver.wsm" }] }),
        json!({ "books": [{ "manuscript": "../Gulliver.wsm" }, { "manuscript": "../Gulliver.wsm" }] }),
        json!({ "books": [{ "manuscript": "../Gulliver.wsm" }, { "manuscript": "../Nowhere.wsm" }] }),
    ] {
        assert!(app.command(commands::BOX_SET, &bad).is_err(), "{bad}");
    }
    app.book.as_mut().unwrap().save().unwrap();
    let mut again = BookApp::new();
    again.open_path(&path.display().to_string()).unwrap();
    assert_eq!(again.book.as_ref().unwrap().files.project.box_set.len(), 2);
    assert_eq!(again.book.as_ref().unwrap().source.as_ref().unwrap().book.body.len(), 2);
    assert!(app.book.as_mut().unwrap().undo());
    let b = app.book.as_ref().unwrap();
    assert!(b.files.project.box_set.is_empty());
    assert_eq!(b.source.as_ref().unwrap().book.body.len(), alone, "the one book again");
}
