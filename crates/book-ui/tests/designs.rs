// SPDX-License-Identifier: MIT OR Apache-2.0
//! Designs in the app (plan M4.3): choose, mix a part, save as the user's own, export and import
//! one file exactly; each change one undo step.

use std::path::{Path, PathBuf};

use serde_json::json;
use wordsmith_book_ui::{BookApp, commands, designs};

fn app(name: &str) -> (BookApp, PathBuf) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("designs").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut a = BookApp::new();
    a.command(commands::NEW, &json!({ "path": dir.join("B.wsbk").display().to_string() })).unwrap();
    (a, dir)
}

#[test]
fn choose_mix_save_export_and_import() {
    let (mut a, dir) = app("all");
    let built_in = wordsmith_book_design::builtin::FILES.len();
    assert_eq!(a.command(commands::DESIGNS, &json!({})).unwrap().as_array().unwrap().len(), built_in);
    a.command(commands::DESIGN_CHOOSE, &json!({ "id": "literary" })).unwrap();
    a.command(commands::DESIGN_MIX, &json!({ "part": "chapter-opening", "from": "fantasy" })).unwrap();
    let d = designs::current(a.book.as_ref().unwrap()).unwrap();
    assert_eq!(d.chapter.font, "inscription");
    assert_eq!(d.body.font, "reading-serif");
    assert!(a.command(commands::DESIGN_MIX, &json!({ "part": "colour", "from": "fantasy" })).is_err());
    assert!(a.command(commands::DESIGN_CHOOSE, &json!({ "id": "nope" })).is_err());
    // Saved as the user's own: the book uses it, it is in the project, and the list has it.
    let id = a.command(commands::DESIGN_SAVE_AS, &json!({ "name": "Story Time" })).unwrap()["id"].as_str().unwrap().to_owned();
    assert_eq!(id, "story-time");
    assert_eq!(designs::current(a.book.as_ref().unwrap()).unwrap().chapter.font, "inscription");
    assert_eq!(a.command(commands::DESIGNS, &json!({})).unwrap().as_array().unwrap().len(), built_in + 1);
    // Export, then import into another book: the same design.
    let file = dir.join("story-time.toml");
    a.command(commands::DESIGN_EXPORT, &json!({ "path": file.display().to_string() })).unwrap();
    let mine = designs::current(a.book.as_ref().unwrap()).unwrap();
    let (mut other, _) = app("other");
    let id2 = other.command(commands::DESIGN_IMPORT, &json!({ "path": file.display().to_string() })).unwrap()["id"].as_str().unwrap().to_owned();
    other.command(commands::DESIGN_CHOOSE, &json!({ "id": id2 })).unwrap();
    assert_eq!(designs::current(other.book.as_ref().unwrap()).unwrap(), mine);
    // Each was one undo step.
    a.command("edit.undo", &json!({})).unwrap();
    assert_eq!(a.book.as_ref().unwrap().files.project.design.id, "literary");
    // Saved and reopened, the user's design is in designs/.
    a.command("edit.redo", &json!({})).unwrap();
    a.command("file.save", &json!({})).unwrap();
    assert!(a.book.as_ref().unwrap().root.join("designs/story-time.toml").exists());
}
