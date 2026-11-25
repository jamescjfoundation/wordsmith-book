// SPDX-License-Identifier: MIT OR Apache-2.0
//! Re-import (plan R2.12, IMP-8): a new version of the book's file is compared word by word with
//! the book's own source; taking it keeps the design, metadata, front and back matter and each
//! section's settings.

use std::path::{Path, PathBuf};

use serde_json::json;
use wordsmith_book_ui::{BookApp, commands};

fn tmp(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_new_version_is_compared_and_taken_with_the_books_own_layer_kept() {
    let dir = tmp("reimport");
    let original = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels/gullivers-travels.txt")).unwrap();
    let v1 = dir.join("gulliver.txt");
    std::fs::write(&v1, &original).unwrap();
    let mut app = BookApp::new();
    app.command(commands::IMPORT, &json!({ "file": v1.display().to_string(), "path": dir.join("G.wsbk").display().to_string() })).unwrap();
    // Book's own layer.
    app.command(commands::DESIGN_CHOOSE, &json!({ "id": "fantasy" })).unwrap();
    app.command(commands::METADATA_SET, &json!({ "field": "imprint", "value": "Harbour Lights Press" })).unwrap();
    let ded = app.command(commands::ADD_ELEMENT, &json!({ "kind": "dedication" })).unwrap()["id"].as_str().unwrap().to_owned();
    app.command(commands::SET_ELEMENT, &json!({ "id": ded, "text": "For the sailors." })).unwrap();
    let chapter = app.book.as_ref().unwrap().source.as_ref().unwrap().book.body.iter().flat_map(|p| p.children.iter()).nth(1).unwrap().clone();
    app.command(commands::SECTION, &json!({ "id": chapter.id.as_text(), "subtitle": "In Which He Is Fed" })).unwrap();
    // The same file: nothing to take.
    let same = app.command(commands::REIMPORT, &json!({ "file": v1.display().to_string() })).unwrap();
    assert_eq!(same["same"], true, "{same}");
    // A new version: one word changed, a sentence added.
    let v2 = dir.join("gulliver-2.txt");
    let changed = original.replacen("Nottinghamshire", "Northamptonshire", 1).replacen(
        "My father had a small estate",
        "We were a loud house.\n\nMy father had a small estate",
        1,
    );
    assert_ne!(changed, original);
    std::fs::write(&v2, &changed).unwrap();
    let before = app.book.as_ref().unwrap().source.as_ref().unwrap().book.clone();
    let r = app.command(commands::REIMPORT, &json!({ "file": v2.display().to_string() })).unwrap();
    assert_eq!(r["same"], false);
    let lines: Vec<String> = r["sections"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| s["changes"].as_array().unwrap().iter().map(|l| l.as_str().unwrap().to_owned()))
        .collect();
    assert!(lines.iter().any(|l| l == "- Nottinghamshire") && lines.iter().any(|l| l == "+ Northamptonshire"), "{lines:?}");
    assert!(lines.iter().any(|l| l.starts_with('+') && l.contains("We were a loud house")), "{lines:?}");
    assert_eq!(r["added"].as_u64().unwrap(), 6);
    assert_eq!(r["removed"].as_u64().unwrap(), 1);
    // Looking does not change the book.
    assert_eq!(app.book.as_ref().unwrap().source.as_ref().unwrap().book, before);
    // Taking it.
    app.command(commands::REIMPORT, &json!({ "file": v2.display().to_string(), "apply": true })).unwrap();
    let b = app.book.as_ref().unwrap();
    let words = wordsmith_book_check::verify::book_words(&b.source.as_ref().unwrap().book, true).join(" ");
    // The word appears three times; the first was changed.
    assert!(words.contains("Northamptonshire") && words.contains("loud house") && words.matches("Nottinghamshire").count() == 2);
    let p = &b.files.project;
    assert_eq!(p.design.id, "fantasy");
    assert_eq!(p.metadata.imprint.as_ref().unwrap().name, "Harbour Lights Press");
    assert!(p.elements.iter().any(|e| e.id == ded && e.text == "For the sailors."));
    // The chapter's subtitle followed it to the new version of the section.
    let new_chapter = b.source.as_ref().unwrap().book.body.iter().flat_map(|p| p.children.iter()).nth(1).unwrap();
    assert_eq!(p.section(new_chapter.id).subtitle, "In Which He Is Fed");
    // Undo puts the old text back.
    assert!(app.book.as_mut().unwrap().undo_owned().unwrap());
    let back = wordsmith_book_check::verify::book_words(&app.book.as_ref().unwrap().source.as_ref().unwrap().book, true).join(" ");
    assert_eq!(back.matches("Nottinghamshire").count(), 3);
}

#[test]
fn a_linked_manuscript_is_not_reimported() {
    let dir = tmp("reimport-linked");
    let mut app = BookApp::new();
    let ms = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels/gullivers-travels.wsm");
    app.command(commands::NEW, &json!({ "path": dir.join("B.wsbk").display().to_string(), "manuscript": ms.display().to_string() })).unwrap();
    let txt = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels/gullivers-travels.txt");
    let err = app.command(commands::REIMPORT, &json!({ "file": txt.display().to_string() })).unwrap_err();
    assert!(err.contains("Manuscript"), "{err}");
}
