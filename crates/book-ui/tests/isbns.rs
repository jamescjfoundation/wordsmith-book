// SPDX-License-Identifier: MIT OR Apache-2.0
//! The house's ISBN list (plan R4.4, META-5): each ISBN in the house file is checked (its check
//! digit) and owned once; an edition takes the next free one, and the house file records which
//! book and edition use it, so no ISBN is used twice; a list with a bad or repeated ISBN is
//! refused.

use std::path::{Path, PathBuf};

use serde_json::json;
use wordsmith_book_ui::{BookApp, commands};

const LIST: &str = r#"
schema = 1

[[isbns]]
isbn = "978-0-306-40615-7"
book = "An Older Book"
edition = "Paperback"

[[isbns]]
isbn = "978-1-86197-271-2"

[[isbns]]
isbn = "9780000000019"
"#;

fn book(dir: &Path, name: &str, title: &str) -> BookApp {
    let file = dir.join(format!("{name}.txt"));
    std::fs::write(&file, format!("{title}\n\nCHAPTER I.\n\nThe lamp burned low over the harbour.\n")).unwrap();
    let mut app = BookApp::new();
    app.command(commands::IMPORT, &json!({ "file": file.display().to_string(), "path": dir.join(format!("{name}.wsbk")).display().to_string() }))
        .unwrap();
    app.command(commands::METADATA_SET, &json!({ "field": "title", "value": title })).unwrap();
    app
}

fn set_up(name: &str) -> (PathBuf, PathBuf) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let house = dir.join("house.toml");
    std::fs::write(&house, LIST).unwrap();
    (dir, house)
}

#[test]
fn editions_take_free_isbns_each_once() {
    let (dir, house) = set_up("isbn-assign");
    let h = house.display().to_string();
    let mut a = book(&dir, "a", "The Harbour Lamp");
    let v = a.command(commands::ISBN_ASSIGN, &json!({ "house": h, "edition": "paperback" })).unwrap();
    assert_eq!(v["isbn"], "978-1-86197-271-2", "the first free one, as the house writes it");
    assert_eq!(a.book.as_ref().unwrap().files.project.edition("paperback").unwrap().isbn, "978-1-86197-271-2");
    // An edition with an ISBN does not take another.
    assert!(a.command(commands::ISBN_ASSIGN, &json!({ "house": h, "edition": "paperback" })).is_err());
    a.command(commands::ISBN_ASSIGN, &json!({ "house": h, "edition": "ebook" })).unwrap();
    // The house file records the uses; another book finds none free.
    let list = a.command(commands::ISBN_LIST, &json!({ "house": h })).unwrap();
    let rows: Vec<(String, String, String)> = list["isbns"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["isbn"].as_str().unwrap().to_owned(), r["book"].as_str().unwrap().to_owned(), r["edition"].as_str().unwrap().to_owned()))
        .collect();
    assert_eq!(rows[1], ("978-1-86197-271-2".into(), "The Harbour Lamp".into(), "Paperback".into()));
    assert_eq!(rows[2], ("9780000000019".into(), "The Harbour Lamp".into(), "Ebook".into()));
    let mut b = book(&dir, "b", "Another Tide");
    let e = b.command(commands::ISBN_ASSIGN, &json!({ "house": h, "edition": "paperback" })).unwrap_err();
    assert!(e.contains("no free ISBN"), "{e}");
    assert!(b.book.as_ref().unwrap().files.project.edition("paperback").unwrap().isbn.is_empty());
    // Undoing the book's change leaves the house's record: the ISBN stays the book's, not free.
    assert!(a.book.as_mut().unwrap().undo());
}

#[test]
fn a_list_with_a_bad_or_repeated_isbn_is_refused() {
    let (dir, house) = set_up("isbn-bad");
    let h = house.display().to_string();
    let mut a = book(&dir, "a", "The Harbour Lamp");
    for bad in [
        "schema = 1\n[[isbns]]\nisbn = \"978-0-306-40615-8\"\n",
        "schema = 1\n[[isbns]]\nisbn = \"978-0-306-40615-7\"\n[[isbns]]\nisbn = \"9780306406157\"\n",
        "schema = 1\n[[isbns]]\nisbn = \"9780000000019\"\nbook = \"X\"\nedition = \"Ebook\"\n[[isbns]]\nisbn = \"9780000000026\"\nbook = \"X\"\nedition = \"Ebook\"\n",
    ] {
        std::fs::write(&house, bad).unwrap();
        let e = a.command(commands::ISBN_LIST, &json!({ "house": h })).unwrap_err();
        assert!(e.contains("ISBN"), "{e}");
        assert!(a.command(commands::ISBN_ASSIGN, &json!({ "house": h, "edition": "paperback" })).is_err());
    }
}
