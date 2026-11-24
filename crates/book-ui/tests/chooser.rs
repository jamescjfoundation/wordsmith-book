// SPDX-License-Identifier: MIT OR Apache-2.0
//! The design chooser (plan M4.4): filters, the book's own first chapter in the preview, and
//! choosing through the harness as one undo step.

use std::path::Path;

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use jcj_shell::egui;
use jcj_shell::Shell;
use serde_json::json;
use wordsmith_book_design::builtin;
use wordsmith_book_ui::chooser::{filter, first_chapter, tags};
use wordsmith_book_ui::{BookApp, NAME, brand, commands, shell_options};

#[test]
fn filters_by_genre_and_mood() {
    let all = builtin::all();
    let ids = |v: Vec<&wordsmith_book_design::Design>| v.iter().map(|d| d.id.clone()).collect::<Vec<_>>();
    assert_eq!(filter(&all, None, None).len(), 6);
    assert_eq!(ids(filter(&all, Some("fantasy"), None)), ["fantasy"]);
    assert_eq!(ids(filter(&all, Some("historical"), None)), ["classic", "romance", "fantasy"]);
    assert_eq!(ids(filter(&all, Some("historical"), Some("epic"))), ["fantasy"]);
    assert!(filter(&all, Some("cookbook"), None).is_empty());
    let (genres, moods) = tags(&all);
    assert!(genres.contains(&"thriller".to_owned()) && moods.contains(&"tense".to_owned()));
}

struct St {
    shell: Shell,
    app: BookApp,
}

#[test]
fn the_chooser_previews_the_book_and_chooses_in_one_undo_step() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("chooser");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("tale.txt");
    std::fs::write(&file, "CHAPTER I.\n\nThe lamp burned low over the harbour.\n").unwrap();
    let mut app = BookApp::new();
    app.command(commands::IMPORT, &json!({ "file": file.display().to_string(), "path": dir.join("T.wsbk").display().to_string() })).unwrap();
    assert_eq!(first_chapter(app.book.as_ref().unwrap()).1, "The lamp burned low over the harbour.");
    let shell = Shell::new(NAME, "0.1.0", brand::tokens()).with_options(shell_options());
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 1000.0)).build_ui_state(|ui, st: &mut St| st.shell.ui(ui, &mut st.app), St { shell, app });
    h.run();
    for _ in 0..2 {
        if h.query_by_label("Choose Fantasy").is_none() {
            h.state_mut().shell.run(jcj_shell::Cmd::with(jcj_shell::commands::shell_ids::PANEL, json!({ "panel": "designs" })));
            h.run();
        }
    }
    // Filter to fantasy: only its card shows.
    h.get_by_label("fantasy").click();
    h.run();
    assert!(h.query_by_label("Choose Literary").is_none());
    assert!(h.query_by_label_contains("The lamp burned low").is_some(), "the preview shows the book's own first chapter");
    h.get_by_label("Choose Fantasy").click();
    h.run();
    assert_eq!(h.state().app.book.as_ref().unwrap().files.project.design.id, "fantasy");
    h.state_mut().app.command("edit.undo", &json!({})).unwrap();
    assert_eq!(h.state().app.book.as_ref().unwrap().files.project.design.id, "classic");
}
