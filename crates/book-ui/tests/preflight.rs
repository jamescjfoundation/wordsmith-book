// SPDX-License-Identifier: MIT OR Apache-2.0
//! The preflight panel (plan M8.1): it runs in the background, lists what it found and follows a
//! finding's link; the command gives the same findings to agents.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use jcj_shell::{Shell, egui};
use serde_json::json;
use wordsmith_book_ui::{BookApp, NAME, brand, commands, shell_options};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

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

fn linked_app(name: &str) -> BookApp {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let src = root().join("tests/fixtures/novels/gullivers-travels.wsm");
    let mut files = BTreeMap::new();
    tree(&src, &src, &mut files);
    for (rel, bytes) in files {
        let p = dir.join("Gulliver.wsm").join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    let mut app = BookApp::new();
    app.command(
        commands::NEW,
        &json!({ "path": dir.join("Gulliver.wsbk").display().to_string(), "manuscript": "../Gulliver.wsm", "date": "2026-11-24" }),
    )
    .unwrap();
    app
}

struct St {
    shell: Shell,
    app: BookApp,
}

#[test]
fn preflight_lists_findings_and_follows_their_links() {
    let mut app = linked_app("preflight-panel");
    let first = app.book.as_ref().unwrap().files.project.editions[0].id.clone();
    // A clean paperback: nothing blocks.
    let v = app.command(commands::PREFLIGHT, &json!({ "edition": first })).unwrap();
    assert_eq!(v["blocked"], false, "{v}");
    // A wrong ISBN blocks, linked to its edition.
    // (The details panel refuses one; a hand-edited book.toml can still hold one.)
    assert!(app.command(commands::EDITION_SET, &json!({ "id": first, "isbn": "978-0-306-40615-8" })).is_err());
    app.book
        .as_mut()
        .unwrap()
        .edit(|f| {
            f.project.editions[0].isbn = "978-0-306-40615-8".into();
            Ok(())
        })
        .unwrap();
    let v = app.command(commands::PREFLIGHT, &json!({})).unwrap();
    assert_eq!(v["blocked"], true);
    let isbn = v["findings"].as_array().unwrap().iter().find(|f| f["message"].as_str().unwrap().contains("ISBN")).unwrap().clone();
    assert_eq!(isbn["severity"], "Blocks export");
    assert_eq!(isbn["link"]["edition"], first.as_str());
    // Through the panel.
    let shell = Shell::new(NAME, "0.1.0", brand::tokens()).with_options(shell_options());
    let mut h =
        Harness::builder().with_size(egui::vec2(1600.0, 1000.0)).build_ui_state(|ui, st: &mut St| st.shell.ui(ui, &mut st.app), St { shell, app });
    h.run_steps(2);
    h.state_mut().shell.run(jcj_shell::Cmd::with(jcj_shell::commands::shell_ids::PANEL, json!({ "panel": "preflight" })));
    h.run_steps(2);
    h.get_by_label("Run Preflight").click();
    h.run_steps(2);
    assert!(h.state().app.preflight.running());
    for _ in 0..600 {
        h.step();
        std::thread::sleep(Duration::from_millis(20));
        if !h.state().app.preflight.running() {
            break;
        }
    }
    h.run_steps(2);
    assert!(h.query_by_label_contains("must be fixed before export").is_some());
    assert!(!h.query_all_by_label_contains("ISBN").collect::<Vec<_>>().is_empty());
    h.query_all_by_label("Open Edition Details").next().unwrap().click();
    h.run_steps(3);
    assert!(h.state().shell.layout.is_open("details"), "the edition's details open");
}
