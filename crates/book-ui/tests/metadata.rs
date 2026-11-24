// SPDX-License-Identifier: MIT OR Apache-2.0
//! Metadata and editions (plan M3.3): ISBNs checked, Book's additions kept in book.toml and never
//! written to a linked source, and what the book needs listed.

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
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("metadata").join(name);
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

#[test]
fn isbns_are_checked_and_additions_never_reach_a_linked_source() {
    let (mut app, ms) = linked("additions");
    let before = tree(&ms);
    // A wrong check digit is refused with the digit it should be.
    let e = app.command(commands::EDITION_SET, &json!({ "id": "paperback", "isbn": "978-0-306-40615-8" })).unwrap_err();
    assert!(e.contains("check digit") && e.contains('7'), "{e}");
    app.command(commands::EDITION_SET, &json!({ "id": "paperback", "isbn": "978-0-306-40615-7", "paper": "white" })).unwrap();
    // An old ISBN-10 is accepted too.
    let hc = app.command(commands::EDITION_ADD, &json!({ "format": "hardcover" })).unwrap()["id"].as_str().unwrap().to_owned();
    app.command(commands::EDITION_SET, &json!({ "id": hc, "isbn": "0-8044-2957-X" })).unwrap();
    // Book's additions.
    app.command(commands::METADATA_SET, &json!({ "field": "imprint", "value": "Harbour Light Books" })).unwrap();
    app.command(commands::METADATA_SET, &json!({ "field": "subtitle", "value": "A New Edition" })).unwrap();
    app.command(commands::METADATA_SET, &json!({ "field": "subject_codes", "value": "FIC019000, FIC004000" })).unwrap();
    app.command(commands::CONTRIBUTOR_ADD, &json!({ "name": "A. Editor", "role": "editor" })).unwrap();
    assert!(app.command(commands::CONTRIBUTOR_ADD, &json!({ "name": "X", "role": "sorcerer" })).is_err());
    assert!(app.command(commands::METADATA_SET, &json!({ "field": "colour", "value": "red" })).is_err());
    app.command("file.save", &json!({})).unwrap();
    let saved = wordsmith_book_format::Folder::open(&app.book.as_ref().unwrap().root).unwrap().1.project;
    assert_eq!(saved.metadata.subject_codes, ["FIC019000", "FIC004000"]);
    assert_eq!(saved.metadata.overrides.get("subtitle").map(String::as_str), Some("A New Edition"));
    assert_eq!(saved.edition("paperback").unwrap().isbn, "978-0-306-40615-7");
    assert_eq!(tree(&ms), before, "Book's additions are never written to the linked project");
}

#[test]
fn needs_list_the_sources_missing_metadata_and_books_own() {
    let (mut app, _) = linked("needs");
    let m = app.command(commands::METADATA, &json!({})).unwrap();
    let needs: Vec<String> = m["needs"].as_array().unwrap().iter().map(|n| n["message"].as_str().unwrap().to_owned()).collect();
    // The fixture sets no copyright year; no edition has an ISBN.
    assert!(needs.iter().any(|n| n.contains("copyright year")), "{needs:?}");
    assert!(needs.iter().any(|n| n.contains("paperback has no ISBN")), "{needs:?}");
    // The same ISBN on two editions blocks export; a printed edition without a trim does too.
    app.command(commands::EDITION_SET, &json!({ "id": "paperback", "isbn": "9780306406157", "trim": "" })).unwrap();
    app.command(commands::EDITION_SET, &json!({ "id": "ebook", "isbn": "978-0-306-40615-7" })).unwrap();
    let m = app.command(commands::METADATA, &json!({})).unwrap();
    let blocks: Vec<String> = m["needs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["severity"] == "Blocks export")
        .map(|n| n["message"].as_str().unwrap().to_owned())
        .collect();
    assert!(blocks.iter().any(|n| n.contains("same ISBN")), "{blocks:?}");
    assert!(blocks.iter().any(|n| n.contains("no trim size")), "{blocks:?}");
}
