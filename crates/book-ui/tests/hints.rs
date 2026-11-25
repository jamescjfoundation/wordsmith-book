// SPDX-License-Identifier: MIT OR Apache-2.0
//! Line-break hints (plan R2.15, TXT-5): set through the command and the Details panel, refused
//! when unusable, undone, saved with the project and given to agents.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use jcj_shell::{Shell, egui};
use serde_json::json;
use wordsmith_book_ui::{BookApp, NAME, brand, commands, shell_options};

fn tree(dir: &Path, base: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            tree(&p, base, out);
        } else {
            out.insert(p.strip_prefix(base).unwrap().to_owned(), std::fs::read(&p).unwrap());
        }
    }
}

fn linked_app(name: &str) -> (BookApp, PathBuf) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels/gullivers-travels.wsm");
    let mut files = BTreeMap::new();
    tree(&src, &src, &mut files);
    for (rel, bytes) in files {
        let p = dir.join("Gulliver.wsm").join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    let mut app = BookApp::new();
    let path = dir.join("Gulliver.wsbk");
    app.command(commands::NEW, &json!({ "path": path.display().to_string(), "manuscript": "../Gulliver.wsm", "date": "2026-11-24" })).unwrap();
    (app, path)
}

#[test]
fn hints_are_set_refused_undone_and_saved() {
    let (mut app, path) = linked_app("hints-command");
    assert_eq!(app.command(commands::HINTS, &json!({})).unwrap(), json!({ "keep": [], "hyphenate": [] }));
    let v = app.command(commands::HINTS_SET, &json!({ "keep": ["Mr. Smith", "  "], "hyphenate": ["Lil-li-put"] })).unwrap();
    assert_eq!(v, json!({ "keep": ["Mr. Smith"], "hyphenate": ["Lil-li-put"] }), "blank entries are dropped");
    // A list left out is kept.
    app.command(commands::HINTS_SET, &json!({ "keep": ["Captain Gulliver"] })).unwrap();
    assert_eq!(app.command(commands::HINTS, &json!({})).unwrap(), json!({ "keep": ["Captain Gulliver"], "hyphenate": ["Lil-li-put"] }));
    // Unusable hints are refused and change nothing.
    for bad in [json!({ "hyphenate": ["two words"] }), json!({ "hyphenate": ["-x"] }), json!({ "keep": "Mr. Smith" })] {
        assert!(app.command(commands::HINTS_SET, &bad).is_err(), "{bad}");
    }
    assert_eq!(app.command(commands::HINTS, &json!({})).unwrap()["keep"], json!(["Captain Gulliver"]));
    assert!(app.book.as_mut().unwrap().undo());
    assert_eq!(app.command(commands::HINTS, &json!({})).unwrap()["keep"], json!(["Mr. Smith"]));
    app.book.as_mut().unwrap().save().unwrap();
    let mut again = BookApp::new();
    again.open_path(&path.display().to_string()).unwrap();
    assert_eq!(again.command(commands::HINTS, &json!({})).unwrap(), json!({ "keep": ["Mr. Smith"], "hyphenate": ["Lil-li-put"] }));
}

struct St {
    shell: Shell,
    app: BookApp,
}

#[test]
fn the_details_panel_edits_the_hints() {
    let (mut app, _) = linked_app("hints-panel");
    app.command(commands::HINTS_SET, &json!({ "keep": ["Mr. Smith"] })).unwrap();
    let shell = Shell::new(NAME, "0.1.0", brand::tokens()).with_options(shell_options());
    let mut h =
        Harness::builder().with_size(egui::vec2(1600.0, 1200.0)).build_ui_state(|ui, st: &mut St| st.shell.ui(ui, &mut st.app), St { shell, app });
    h.run_steps(2);
    h.state_mut().shell.run(jcj_shell::Cmd::with(jcj_shell::commands::shell_ids::PANEL, json!({ "panel": "details" })));
    h.run_steps(2);
    let keep = h.get_by_label("Never break across lines (one word or phrase a line)");
    assert_eq!(keep.value().as_deref(), Some("Mr. Smith"));
    // Type a word into the hyphenation list, then leave it.
    h.get_by_label("Hyphenate only here (one word a line, a hyphen where it may break; none: never)").focus();
    h.run_steps(2);
    h.get_by_label("Hyphenate only here (one word a line, a hyphen where it may break; none: never)").type_text("Brob-ding-nag");
    h.run_steps(2);
    h.get_by_label("Never break across lines (one word or phrase a line)").focus();
    h.run_steps(3);
    let v = h.state_mut().app.command(commands::HINTS, &json!({})).unwrap();
    assert_eq!(v, json!({ "keep": ["Mr. Smith"], "hyphenate": ["Brob-ding-nag"] }));
}
