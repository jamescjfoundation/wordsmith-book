// SPDX-License-Identifier: MIT OR Apache-2.0
//! Light fixes in the app (plan M3.4): a linked source gets offers and its folder is untouched;
//! a book's own source is fixed in place; fixes list where they stand.

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

fn tmp(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("fixes-app").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_linked_source_gets_an_offer_and_stays_untouched() {
    let dir = tmp("linked");
    let ms = dir.join("Gulliver.wsm");
    for (rel, bytes) in tree(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels/gullivers-travels.wsm")) {
        let p = ms.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    let before = tree(&ms);
    let mut app = BookApp::new();
    app.command(commands::NEW, &json!({ "path": dir.join("B.wsbk").display().to_string(), "manuscript": "../Gulliver.wsm" })).unwrap();
    let ch = app.book.as_ref().unwrap().source.as_ref().unwrap().book.body[0].children[0].id;
    let r = app.command(commands::FIX_EDIT, &json!({ "section": ch.as_text(), "paragraph": 1, "text": "My father had a little estate in Nottinghamshire; I was the third of five sons.", "note": "a test" })).unwrap();
    assert!(r["find"].as_str().unwrap().contains("small"), "{r}");
    let offers = wordsmith_manuscript_format::offers::read(&std::fs::read(wordsmith_manuscript_format::offers::offers_path(&ms)).unwrap()).unwrap();
    assert_eq!(offers.offers.len(), 1);
    assert_eq!(tree(&ms), before, "the project folder is untouched");
    let list = app.command(commands::FIX_LIST, &json!({})).unwrap();
    assert!(list[0]["state"].as_str().unwrap().contains("Not yet accepted"), "{list}");
    // A fix whose words are not there once is refused before anything is written.
    assert!(app.command(commands::FIX_ADD, &json!({ "section": ch.as_text(), "find": "the", "replace": "a" })).is_err());
    assert_eq!(wordsmith_manuscript_format::offers::read(&std::fs::read(wordsmith_manuscript_format::offers::offers_path(&ms)).unwrap()).unwrap().offers.len(), 1);
}

#[test]
fn a_books_own_source_is_fixed_in_place() {
    let dir = tmp("owned");
    let file = dir.join("story.txt");
    std::fs::write(&file, "CHAPTER I.\n\nTeh harbour was still.\n").unwrap();
    let mut app = BookApp::new();
    app.command(commands::IMPORT, &json!({ "file": file.display().to_string(), "path": dir.join("S.wsbk").display().to_string() })).unwrap();
    let ch = app.book.as_ref().unwrap().source.as_ref().unwrap().book.body[0].id;
    app.command(commands::FIX_ADD, &json!({ "section": ch.as_text(), "find": "Teh", "replace": "The" })).unwrap();
    let b = &app.book.as_ref().unwrap().source.as_ref().unwrap().book;
    let text = wordsmith_manuscript_model::Text { blocks: b.body[0].content.clone(), notes: vec![] }.plain_paragraphs();
    assert_eq!(text, ["The harbour was still."]);
    assert!(app.command(commands::FIX_LIST, &json!({})).unwrap()[0]["state"].as_str().unwrap().contains("Made"));
    // The cleanup undo puts the source back.
    app.command(commands::CLEANUP_UNDO, &json!({})).unwrap();
}
