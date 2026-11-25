// SPDX-License-Identifier: MIT OR Apache-2.0
//! The cover commands (plan R2.11): a print edition's template written, a cover checked.

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
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cover").join(name);
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
fn the_cover_template_is_written_and_a_cover_checked() {
    let (mut app, ms) = linked("cover");
    let out = ms.parent().unwrap().join("covers");
    let v = app.command(commands::COVER_TEMPLATE, &json!({ "edition": "paperback", "folder": out.display().to_string() })).unwrap();
    let files: Vec<String> = v["files"].as_array().unwrap().iter().map(|f| f.as_str().unwrap().to_owned()).collect();
    assert_eq!(files.len(), 2);
    assert!(files.iter().all(|f| Path::new(f).is_file()));
    assert_eq!(v["pages"].as_u64().unwrap() % 2, 0, "an even page count");
    // The template's own PDF passes; a small image does not.
    let ok = app.command(commands::COVER_CHECK, &json!({ "edition": "paperback", "file": files[0] })).unwrap();
    assert_eq!(ok["ok"], true, "{ok}");
    let bad = app.command(commands::COVER_CHECK, &json!({ "edition": "paperback", "file": files[1] })).unwrap();
    assert_eq!(bad["ok"], false, "the 100-pixel-per-inch guide is too coarse to print");
    assert!(app.command(commands::COVER_TEMPLATE, &json!({ "edition": "ebook", "folder": out.display().to_string() })).is_err());
}
