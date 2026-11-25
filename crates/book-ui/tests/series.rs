// SPDX-License-Identifier: MIT OR Apache-2.0
//! Series (plan R4.3, META-4): a book that joins a series as its second book gets the series line
//! on its copyright page and "also by" built from the series' other books; the series view lists
//! the books in order with the design each uses and finds one off the series' design.

use std::path::{Path, PathBuf};

use serde_json::json;
use wordsmith_book_model::ElementKind;
use wordsmith_book_ui::{BookApp, commands};

const HOUSE: &str = r#"
schema = 1

[[imprints]]
id = "open"
name = "Open Door"

[[series]]
id = "voyages"
name = "Voyages"
imprint = "open"
design = "classic"
books = [
    { number = 1, title = "The First Voyage", project = "books/first.wsbk" },
    { number = 2, title = "The Second Voyage" },
    { number = 3, title = "The Third Voyage", project = "books/third.wsbk" },
]
"#;

/// A book with its own source and `design`, saved at `path`.
fn book_at(path: &Path, design: &str) {
    let mut app = BookApp::new();
    app.command(commands::NEW, &json!({ "path": path.display().to_string(), "date": "2026-11-24" })).unwrap();
    app.command(commands::DESIGN_CHOOSE, &json!({ "id": design })).unwrap();
    app.book.as_mut().unwrap().save().unwrap();
}

fn set_up(name: &str) -> (BookApp, PathBuf, PathBuf) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("house/books")).unwrap();
    let house = dir.join("house/house.toml");
    std::fs::write(&house, HOUSE).unwrap();
    book_at(&dir.join("house/books/first.wsbk"), "classic");
    book_at(&dir.join("house/books/third.wsbk"), "thriller");
    let file = dir.join("tale.txt");
    std::fs::write(&file, "CHAPTER I.\n\nThe lamp burned low over the harbour, and the ships slept.\n").unwrap();
    let mut app = BookApp::new();
    app.command(commands::IMPORT, &json!({ "file": file.display().to_string(), "path": dir.join("second.wsbk").display().to_string() })).unwrap();
    (app, house, dir)
}

#[test]
fn a_series_book_gets_its_series_line_and_also_by_and_the_view_checks_one_design() {
    let (mut app, house, dir) = set_up("series-view");
    app.command(commands::HOUSE_JOIN, &json!({ "house": house.display().to_string(), "series": "voyages", "number": 2 })).unwrap();
    let b = app.book.as_ref().unwrap();
    assert_eq!(b.files.project.design.id, "classic", "the series' design");
    let also: Vec<(&str, &str)> = b.files.project.metadata.also_by.iter().map(|o| (o.title.as_str(), o.series.as_str())).collect();
    assert_eq!(also, [("The First Voyage", "Voyages"), ("The Third Voyage", "Voyages")]);
    let src = b.source.as_ref().unwrap();
    let a = wordsmith_book_format::assemble::assemble(&b.files.project, &src.book, b.files.project.edition("paperback"));
    let text = |k: ElementKind| -> Vec<String> {
        a.pieces()
            .find(|p| matches!(p.what, wordsmith_book_format::assemble::What::Element(x) if x == k))
            .map(|p| p.blocks.iter().map(wordsmith_book_format::assemble::block_text).collect())
            .unwrap_or_default()
    };
    assert!(text(ElementKind::Copyright).iter().any(|l| l == "Voyages, Book 2"), "{:?}", text(ElementKind::Copyright));
    assert_eq!(text(ElementKind::AlsoBy), ["Voyages", "The First Voyage", "The Third Voyage"]);
    // The series view: the books in order, the third off the series' design.
    let v = app.command(commands::SERIES_VIEW, &json!({ "house": house.display().to_string() })).unwrap();
    assert_eq!(v["series"], "Voyages");
    let rows = v["books"].as_array().unwrap();
    let got: Vec<(u64, &str, bool)> =
        rows.iter().map(|r| (r["number"].as_u64().unwrap(), r["design"].as_str().unwrap(), r["matches"].as_bool().unwrap())).collect();
    assert_eq!(got, [(1, "classic", true), (2, "classic", true), (3, "thriller", false)]);
    assert_eq!(v["one_design"], false);
    // Put the third book on the series' design: one design across the books.
    book_at(&dir.join("house/books/third.wsbk.tmp"), "classic");
    std::fs::remove_dir_all(dir.join("house/books/third.wsbk")).unwrap();
    std::fs::rename(dir.join("house/books/third.wsbk.tmp"), dir.join("house/books/third.wsbk")).unwrap();
    let v = app.command(commands::SERIES_VIEW, &json!({ "house": house.display().to_string() })).unwrap();
    assert_eq!(v["one_design"], true, "{v}");
    // A book in no series has no series view.
    app.command(commands::HOUSE_LEAVE, &json!({})).unwrap();
    assert!(app.command(commands::SERIES_VIEW, &json!({ "house": house.display().to_string() })).is_err());
}
