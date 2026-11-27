// SPDX-License-Identifier: MIT OR Apache-2.0
//! House styles (plan R4.1, DSN-7): a book that joins an imprint or series takes its house
//! design; when the imprint or series locks it, the design cannot be changed in that book by any
//! command, even with the house file gone; leaving frees it.

use std::path::{Path, PathBuf};

use serde_json::json;
use wordsmith_book_ui::{BookApp, commands};

const HOUSE: &str = r#"
schema = 1
name = "Lantern House"

[[imprints]]
id = "lantern"
name = "Lantern Press"
design = "classic"
lock_design = true

[[imprints]]
id = "open"
name = "Open Door"
design = "literary"

[[series]]
id = "voyages"
name = "Voyages"
imprint = "open"
design = "voyage-house"
lock_design = true
"#;

fn set_up(name: &str) -> (BookApp, PathBuf, PathBuf) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("house/designs")).unwrap();
    let house = dir.join("house/house.toml");
    std::fs::write(&house, HOUSE).unwrap();
    // The series' own design, a copy of Fantasy under its own id.
    let fantasy = wordsmith_book_design::builtin::FILES.iter().find(|(id, _)| *id == "fantasy").unwrap().1;
    std::fs::write(
        dir.join("house/designs/voyage-house.toml"),
        fantasy.replace("id = \"fantasy\"", "id = \"voyage-house\"").replace("name = \"Fantasy\"", "name = \"Voyage House\""),
    )
    .unwrap();
    let file = dir.join("tale.txt");
    std::fs::write(&file, "CHAPTER I.\n\nThe lamp burned low over the harbour.\n").unwrap();
    let mut app = BookApp::new();
    let path = dir.join("T.wsbk");
    app.command(commands::IMPORT, &json!({ "file": file.display().to_string(), "path": path.display().to_string() })).unwrap();
    (app, house, path)
}

fn design(app: &BookApp) -> String {
    app.book.as_ref().unwrap().files.project.design.id.clone()
}

fn changes_refused(app: &mut BookApp) {
    for (id, params) in [
        (commands::DESIGN_CHOOSE, json!({ "id": "thriller" })),
        (commands::DESIGN_MIX, json!({ "part": "chapter-opening", "from": "thriller" })),
        (commands::DESIGN_SAVE_AS, json!({ "name": "Mine" })),
    ] {
        let e = app.command(id, &params).unwrap_err();
        assert!(e.contains("house design"), "{id}: {e}");
    }
}

#[test]
fn a_locked_imprint_design_cannot_be_changed_until_the_book_leaves() {
    let (mut app, house, _) = set_up("house-imprint");
    let v = app.command(commands::HOUSE_JOIN, &json!({ "house": house.display().to_string(), "imprint": "lantern" })).unwrap();
    assert_eq!(v["imprint"], "lantern");
    assert_eq!(design(&app), "classic");
    changes_refused(&mut app);
    assert_eq!(design(&app), "classic");
    app.command(commands::HOUSE_LEAVE, &json!({})).unwrap();
    app.command(commands::DESIGN_CHOOSE, &json!({ "id": "thriller" })).unwrap();
    assert_eq!(design(&app), "thriller");
    // An imprint that does not lock its design gives it, and the book may change it.
    app.command(commands::HOUSE_JOIN, &json!({ "house": house.display().to_string(), "imprint": "open" })).unwrap();
    assert_eq!(design(&app), "literary");
    app.command(commands::DESIGN_CHOOSE, &json!({ "id": "romance" })).unwrap();
    // Refused: an imprint or series the house does not have, nothing named, no house file.
    for bad in [
        json!({ "house": house.display().to_string(), "imprint": "nope" }),
        json!({ "house": house.display().to_string() }),
        json!({ "imprint": "open" }),
    ] {
        assert!(app.command(commands::HOUSE_JOIN, &bad).is_err(), "{bad}");
    }
}

#[test]
fn a_series_brings_its_own_house_design_and_keeps_it_without_the_house_file() {
    let (mut app, house, path) = set_up("house-series");
    app.command(commands::HOUSE_JOIN, &json!({ "house": house.display().to_string(), "series": "voyages", "number": 2 })).unwrap();
    let b = app.book.as_ref().unwrap();
    assert_eq!(b.files.project.house.imprint, "open", "the series' imprint comes with it");
    assert_eq!(b.files.project.house.series_number, Some(2));
    assert_eq!(design(&app), "voyage-house");
    assert!(b.files.designs.contains_key("voyage-house"), "the house design is kept in the book");
    assert_eq!(wordsmith_book_ui::designs::current(b).unwrap().chapter.font, "inscription");
    app.book.as_mut().unwrap().save().unwrap();
    std::fs::remove_dir_all(house.parent().unwrap()).unwrap();
    let mut again = BookApp::new();
    again.open_path(&path).unwrap();
    changes_refused(&mut again);
    assert_eq!(design(&again), "voyage-house");
    // Undoing the join (one step) frees the design too.
    assert!(app.book.as_mut().unwrap().undo());
    app.command(commands::DESIGN_CHOOSE, &json!({ "id": "thriller" })).unwrap();
}
