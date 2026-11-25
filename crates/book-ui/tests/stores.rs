// SPDX-License-Identifier: MIT OR Apache-2.0
//! Store editions (plan R2.9, STR-10): ebook editions named by store, each carrying only its own
//! store pages and links.

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
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("stores").join(name);
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

fn files(bytes: &[u8]) -> String {
    wordsmith_book_check::epub::unzip(bytes)
        .unwrap()
        .iter()
        .filter(|f| f.0.ends_with(".xhtml"))
        .map(|f| String::from_utf8_lossy(&f.2).into_owned())
        .collect()
}

#[test]
fn each_store_edition_carries_only_its_own_links() {
    let (mut app, _) = linked("stores");
    let mut eds = Vec::new();
    for store in ["Shop A", "Shop B"] {
        let id = app.command(commands::EDITION_ADD, &json!({ "format": "ebook" })).unwrap()["id"].as_str().unwrap().to_owned();
        app.command(commands::EDITION_SET, &json!({ "id": id, "store": store, "name": format!("Ebook for {store}") })).unwrap();
        eds.push(id);
    }
    for (store, url) in [("Shop A", "https://a.example/book"), ("Shop B", "https://b.example/book")] {
        let id = app.command(commands::ADD_ELEMENT, &json!({ "kind": "also-by", "placement": "back" })).unwrap()["id"].as_str().unwrap().to_owned();
        app.command(
            commands::SET_ELEMENT,
            &json!({ "id": id, "title": "Buy the Next Book", "text": format!("Find the next book at {url}."), "stores": [store] }),
        )
        .unwrap();
    }
    let b = app.book.as_ref().unwrap();
    let p = &b.files.project;
    let book = wordsmith_book_format::fixes::book_for_build(&p.fixes, b.source.as_ref().unwrap());
    let ebook = |id: &str| {
        let e = p.editions.iter().find(|e| e.id == id).unwrap();
        wordsmith_book_check::build::ebook(p, &b.files.designs, &book, e, &|_| None).unwrap().1.bytes
    };
    let (a, bb) = (files(&ebook(&eds[0])), files(&ebook(&eds[1])));
    assert!(a.contains("<a href=\"https://a.example/book\">https://a.example/book</a>.") && !a.contains("b.example"), "Shop A's edition");
    assert!(bb.contains("href=\"https://b.example/book\"") && !bb.contains("a.example"), "Shop B's edition");
    // The plain ebook and the paperback carry neither.
    let plain = files(&ebook("ebook"));
    assert!(!plain.contains("a.example") && !plain.contains("b.example"));
    let pb = p.editions.iter().find(|e| e.format.is_print()).unwrap();
    let pr = wordsmith_book_check::build::print(p, &b.files.designs, &book, pb, &|_| None).unwrap();
    let printed: String = pr.typeset.printed_paragraphs().iter().map(|x| x.0.clone()).collect();
    assert!(!printed.contains("example/book"));
    // Each store edition passes its checks (the links are external, so allowed).
    for id in &eds {
        let v = app.command(commands::PREFLIGHT, &json!({ "edition": id })).unwrap();
        assert_eq!(v["blocked"], false, "{v}");
    }
    assert_eq!(
        wordsmith_book_epub::linked("See https://x.example/a, then go."),
        "See <a href=\"https://x.example/a\">https://x.example/a</a>, then go."
    );
}
