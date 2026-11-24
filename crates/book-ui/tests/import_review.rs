// SPDX-License-Identifier: MIT OR Apache-2.0
//! The import review (plan M2.2) through the egui harness: the online word processor's download
//! shows its chapters and scene breaks; switching a rule off changes what is found; Import makes
//! a book whose owned source opens with every chapter. And the import command without a window.

use std::path::{Path, PathBuf};

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use jcj_shell::egui;
use jcj_shell::{App, Shell};
use serde_json::json;
use wordsmith_book_ui::{BookApp, NAME, brand, commands, shell_options};

fn tmp(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("import").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn fixture(name: &str, dir: &Path) -> PathBuf {
    let to = dir.join(name);
    std::fs::copy(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels").join(name), &to).unwrap();
    to
}

struct St {
    shell: Shell,
    app: BookApp,
}

#[test]
fn the_review_shows_what_was_found_and_imports_it() {
    let dir = tmp("review");
    let file = fixture("gullivers-travels-online.docx", &dir);
    let mut app = BookApp::new();
    app.command(commands::IMPORT_REVIEW, &json!({ "file": file.display().to_string() })).unwrap();
    let shell = Shell::new(NAME, "0.1.0", brand::tokens()).with_options(shell_options());
    let mut h =
        Harness::builder().with_size(egui::vec2(1400.0, 1000.0)).build_ui_state(|ui, st: &mut St| st.shell.ui(ui, &mut st.app), St { shell, app });
    h.run();
    assert!(h.query_by_label_contains("4 parts, 39 chapters").is_some());
    // Switch off the "Chapter 12" rule: the chapters go.
    h.get_by_label_contains("Lines like “Chapter 12”").click();
    h.run();
    assert!(h.query_by_label_contains("4 parts, 0 chapters").is_some());
    h.get_by_label_contains("Lines like “Chapter 12”").click();
    h.run();
    h.get_by_label("Import").click();
    h.run();
    let app = &h.state().app;
    let b = app.book.as_ref().expect("a book was made");
    let src = b.source.as_ref().expect("its owned source opens");
    assert_eq!(src.book.body.len(), 4);
    assert_eq!(src.book.body.iter().map(|p| p.children.len()).sum::<usize>(), 39);
    assert!(app.last_report.as_ref().is_some_and(|r| r.passed()));
}

#[test]
fn the_import_command_makes_a_book_and_gives_its_report() {
    let dir = tmp("command");
    let file = fixture("waverley.odt", &dir);
    let mut app = BookApp::new();
    let target = dir.join("Waverley.wsbk");
    let report = app.command(commands::IMPORT, &json!({ "file": file.display().to_string(), "path": target.display().to_string() })).unwrap();
    assert_eq!(report["passed"], true, "{report}");
    assert_eq!(report["made"]["parts"], 2);
    assert!(report["words"].as_str().unwrap().contains("every word matches"));
    assert!(app.has_document());
    // The rules are kept in book.toml for a re-import.
    let again = wordsmith_book_format::Folder::open(&target).unwrap().1;
    assert_eq!(again.project.cleanup.rules, wordsmith_book_model::ImportRules::default());
    // A second import into the same place is refused.
    assert!(app.command(commands::IMPORT, &json!({ "file": file.display().to_string(), "path": target.display().to_string() })).is_err());
    // A file Book cannot import.
    assert!(app.command(commands::IMPORT_REVIEW, &json!({ "file": dir.join("x.pdf").display().to_string() })).is_err());
}
