// SPDX-License-Identifier: MIT OR Apache-2.0
//! Cleanup in the app (plan M2.4): the list, making all or chosen changes in the book's own
//! source, undoing them, "Keep exactly as written", the panel through the harness, and a linked
//! source left alone.

use std::path::{Path, PathBuf};

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use jcj_shell::Shell;
use jcj_shell::egui;
use serde_json::json;
use wordsmith_book_ui::{BookApp, NAME, brand, commands, shell_options};

fn tmp(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cleanup").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn imported(name: &str) -> BookApp {
    let dir = tmp(name);
    let file = dir.join("messy.txt");
    std::fs::write(&file, "CHAPTER I.\n\n\"Come here,\" she said -- and he came...  He wasn't sure.\n").unwrap();
    let mut app = BookApp::new();
    app.command(commands::IMPORT, &json!({ "file": file.display().to_string(), "path": dir.join("Messy.wsbk").display().to_string() })).unwrap();
    app
}

fn text(app: &BookApp) -> String {
    let b = app.book.as_ref().unwrap().source.as_ref().unwrap();
    b.book
        .body
        .iter()
        .flat_map(|s| s.content.iter())
        .filter_map(|blk| if let wordsmith_manuscript_model::Block::Para(c) = blk { Some(wordsmith_manuscript_model::text::plain(c)) } else { None })
        .collect()
}

#[test]
fn changes_are_made_chosen_undone_or_not_proposed() {
    let mut app = imported("commands");
    let list = app.command(commands::CLEANUP_LIST, &json!({})).unwrap();
    let n = list.as_array().unwrap().len();
    assert!(n >= 5, "{list}");
    // One chosen change.
    let first = list[0]["id"].as_u64().unwrap();
    assert_eq!(app.command(commands::CLEANUP_APPLY, &json!({ "ids": [first] })).unwrap()["made"], 1);
    assert!(text(&app).starts_with('“'));
    app.command(commands::CLEANUP_UNDO, &json!({})).unwrap();
    assert!(text(&app).starts_with('"'));
    // All of them.
    app.command(commands::CLEANUP_APPLY, &json!({})).unwrap();
    assert_eq!(text(&app), "“Come here,” she said — and he came… He wasn’t sure.");
    assert_eq!(app.command(commands::CLEANUP_LIST, &json!({})).unwrap().as_array().unwrap().len(), 0, "nothing left to propose");
    app.command(commands::CLEANUP_UNDO, &json!({})).unwrap();
    // Keep exactly as written: nothing proposed, and that is one undo step.
    app.command(commands::CLEANUP_KEEP, &json!({ "on": true })).unwrap();
    assert_eq!(app.command(commands::CLEANUP_LIST, &json!({})).unwrap().as_array().unwrap().len(), 0);
    app.command("edit.undo", &json!({})).unwrap();
    assert_eq!(app.command(commands::CLEANUP_LIST, &json!({})).unwrap().as_array().unwrap().len(), n);
}

#[test]
fn a_linked_source_is_never_cleaned_up() {
    let dir = tmp("linked");
    let ms = dir.join("Gulliver.wsm");
    let (_, p) = wordsmith_manuscript_format::folder::Folder::open(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels/gullivers-travels.wsm"),
    )
    .unwrap();
    wordsmith_manuscript_format::folder::Folder::create(&ms, &p).unwrap();
    let mut app = BookApp::new();
    app.command(commands::NEW, &json!({ "path": dir.join("B.wsbk").display().to_string(), "manuscript": "../Gulliver.wsm" })).unwrap();
    let e = app.command(commands::CLEANUP_APPLY, &json!({})).unwrap_err();
    assert!(e.contains("Manuscript"), "{e}");
}

struct St {
    shell: Shell,
    app: BookApp,
}

#[test]
fn the_panel_lists_the_changes_and_makes_them() {
    let app = imported("panel");
    let shell = Shell::new(NAME, "0.1.0", brand::BRAND.tokens()).with_options(shell_options());
    let mut h =
        Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_ui_state(|ui, st: &mut St| st.shell.ui(ui, &mut st.app), St { shell, app });
    h.run();
    for _ in 0..2 {
        if h.query_by_label_contains("changes proposed").is_none() {
            h.state_mut().shell.run(jcj_shell::Cmd::with(jcj_shell::commands::shell_ids::PANEL, json!({ "panel": "cleanup" })));
            h.run();
        }
    }
    assert!(h.query_by_label_contains("changes proposed").is_some());
    h.get_by_label("Make Chosen Changes").click();
    h.run();
    assert_eq!(text(&h.state().app), "“Come here,” she said — and he came… He wasn’t sure.");
}
