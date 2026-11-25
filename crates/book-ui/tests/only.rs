// SPDX-License-Identifier: MIT OR Apache-2.0
//! Print-only and ebook-only (plan R2.8, STR-9): a section, an element or a paragraph set to one
//! output is in that output only, and both outputs pass their word checks.

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
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("only").join(name);
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
fn content_set_to_one_output_is_in_that_output_only() {
    let (mut app, _) = linked("only");
    let src = app.book.as_ref().unwrap().source.as_ref().unwrap().book.clone();
    // A whole chapter in print only, a paragraph of another in the ebook only.
    let chapters: Vec<_> = src.body.iter().flat_map(|p| p.children.iter()).collect();
    let print_only = chapters[1].id;
    let ch = chapters[2];
    let para = ch.content.iter().map(wordsmith_book_format::assemble::block_text).find(|t| t.split_whitespace().count() > 12).unwrap();
    let starts: String = para.split_whitespace().take(6).collect::<Vec<_>>().join(" ");
    app.command(commands::SECTION, &json!({ "id": print_only.as_text(), "only": "print" })).unwrap();
    app.command(commands::BLOCK_ONLY, &json!({ "section": ch.id.as_text(), "starts": starts, "only": "ebook" })).unwrap();
    // A dedication in the ebook only.
    let ded = app.command(commands::ADD_ELEMENT, &json!({ "kind": "dedication" })).unwrap()["id"].as_str().unwrap().to_owned();
    app.command(commands::SET_ELEMENT, &json!({ "id": ded, "text": "For every reader on a small screen.", "only": "ebook" })).unwrap();
    // Words that start more than one paragraph are refused.
    assert!(app.command(commands::BLOCK_ONLY, &json!({ "section": ch.id.as_text(), "starts": "The", "only": "print" })).is_err());

    let b = app.book.as_ref().unwrap();
    let p = &b.files.project;
    let book = wordsmith_book_format::fixes::book_for_build(&p.fixes, b.source.as_ref().unwrap());
    let paperback = p.editions.iter().find(|e| e.format.is_print()).unwrap().clone();
    let ebook_ed = p.editions.iter().find(|e| !e.format.is_print()).unwrap().clone();
    let pr = wordsmith_book_check::build::print(p, &b.files.designs, &book, &paperback, &|_| None).unwrap();
    let printed: String = pr.typeset.printed_paragraphs().iter().map(|x| x.0.clone()).collect::<Vec<_>>().join("\n");
    let printed_words = wordsmith_book_check::verify::words(&printed).join(" ");
    assert!(!printed_words.contains(&wordsmith_book_check::verify::words(&starts).join(" ")), "the ebook-only paragraph is not printed");
    assert!(!printed.contains("small screen"), "the ebook-only dedication is not printed");
    assert!(pr.book.sections.iter().any(|s| s.id == print_only.as_text()), "the print-only chapter is printed");
    let (_, e) = wordsmith_book_check::build::ebook(p, &b.files.designs, &book, &ebook_ed, &|_| None).unwrap();
    let files = wordsmith_book_check::epub::unzip(&e.bytes).unwrap();
    let all: String = files.iter().filter(|f| f.0.ends_with(".xhtml")).map(|f| String::from_utf8_lossy(&f.2).into_owned()).collect();
    let words = wordsmith_book_check::epub::words(&e.bytes, false).unwrap().join(" ");
    let want = wordsmith_book_check::verify::words(&starts).join(" ");
    assert!(words.contains(&want), "the ebook-only paragraph is in the ebook");
    assert!(all.contains("small screen"), "the ebook-only dedication is in the ebook");
    // The print-only chapter is not in the ebook: the ebook's word check below compares it with
    // the book without that chapter.
    // Each output passes its word check against the book as it shows it.
    for id in [paperback.id.clone(), ebook_ed.id.clone()] {
        let v = app.command(commands::PREFLIGHT, &json!({ "edition": id })).unwrap();
        let words: Vec<_> = v["findings"].as_array().unwrap().iter().filter(|f| f["rule"] == "words").collect();
        assert!(words.is_empty(), "{id}: {words:?}");
    }
    // Both again: the paragraph is back in print.
    let b = app.book.as_ref().unwrap();
    let ch_id = b.source.as_ref().unwrap().book.body.iter().flat_map(|p| p.children.iter()).nth(2).unwrap().id;
    app.command(commands::BLOCK_ONLY, &json!({ "section": ch_id.as_text(), "starts": starts, "only": "both" })).unwrap();
    assert!(app.book.as_ref().unwrap().files.project.section(ch_id).blocks_only.is_empty());
}
