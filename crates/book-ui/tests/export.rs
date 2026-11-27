// SPDX-License-Identifier: MIT OR Apache-2.0
//! Export (plan M8.3): the editions and the project bundle written twice give the same bytes; a
//! failing check writes nothing; the bundle reopens elsewhere as the same project.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use jcj_shell::{Shell, egui};
use serde_json::json;
use wordsmith_book_ui::{BookApp, NAME, brand, commands, shell_options};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn tmp(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
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

fn linked_app(dir: &Path) -> BookApp {
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
    app.command(commands::DESIGN_CHOOSE, &json!({ "id": "literary" })).unwrap();
    app.book.as_mut().unwrap().save().unwrap();
    app
}

fn read_all(paths: &serde_json::Value) -> BTreeMap<String, Vec<u8>> {
    paths["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            let p = PathBuf::from(p.as_str().unwrap());
            (p.file_name().unwrap().to_string_lossy().into_owned(), std::fs::read(&p).unwrap())
        })
        .collect()
}

#[test]
fn exports_repeat_byte_for_byte_and_the_bundle_reopens_as_the_same_project() {
    let dir = tmp("export");
    let mut app = linked_app(&dir);
    let a = app.command(commands::EXPORT, &json!({ "folder": dir.join("out-a").display().to_string(), "bundle": true })).unwrap();
    let b = app.command(commands::EXPORT, &json!({ "folder": dir.join("out-b").display().to_string(), "bundle": true })).unwrap();
    let (a, b) = (read_all(&a), read_all(&b));
    assert_eq!(a.len(), 3, "{:?}", a.keys().collect::<Vec<_>>());
    assert!(a.keys().any(|k| k.ends_with(".pdf")) && a.keys().any(|k| k.ends_with(".epub")) && a.keys().any(|k| k.ends_with(".wsbkz")));
    for (name, bytes) in &a {
        assert!(b.get(name) == Some(bytes), "{name}: two exports differ");
    }
    // The bundle, opened somewhere the linked manuscript is not, is the same project.
    let elsewhere = tmp("export-elsewhere");
    let (bundle_name, bundle) = a.iter().find(|(k, _)| k.ends_with(".wsbkz")).unwrap();
    let copy = elsewhere.join(bundle_name);
    std::fs::write(&copy, bundle).unwrap();
    let mut other = BookApp::new();
    other.open_path(&copy).unwrap();
    let (x, y) = (app.book.as_ref().unwrap(), other.book.as_ref().unwrap());
    assert_eq!(x.files.project, y.files.project);
    assert_eq!(x.files.designs, y.files.designs);
    let (sx, sy) = (x.source.as_ref().unwrap(), y.source.as_ref().map_err(Clone::clone).unwrap());
    assert_eq!(sx.book, sy.book, "the linked source travels in the bundle");
    // And it exports the same files.
    let again = read_all(&other.command(commands::EXPORT, &json!({ "folder": elsewhere.join("out").display().to_string() })).unwrap());
    for (name, bytes) in again {
        assert!(a.get(&name) == Some(&bytes), "{name}: the reopened bundle exports differently");
    }
}

#[test]
fn a_failing_check_writes_nothing() {
    let dir = tmp("export-fail");
    let mut app = linked_app(&dir);
    // A word the book's fonts cannot print.
    let src = app.book.as_mut().unwrap().source.as_mut().unwrap();
    let first = src.book.body.iter_mut().flat_map(|s| s.children.iter_mut()).find(|s| !s.content.is_empty()).unwrap();
    first.content.insert(
        0,
        wordsmith_manuscript_model::Block::Para(vec![wordsmith_manuscript_model::Inline::Span { text: "東京".into(), marks: Default::default() }]),
    );
    let out = dir.join("out");
    let err = app.command(commands::EXPORT, &json!({ "folder": out.display().to_string(), "bundle": true })).unwrap_err();
    assert!(err.contains("nothing was exported"), "{err}");
    assert!(!out.exists(), "nothing was written");
}

struct St {
    shell: Shell,
    app: BookApp,
}

#[test]
fn the_export_page_names_its_choices() {
    let dir = tmp("export-page");
    let app = linked_app(&dir);
    let shell = Shell::new(NAME, "0.1.0", brand::BRAND.tokens()).with_options(shell_options());
    let mut h =
        Harness::builder().with_size(egui::vec2(1600.0, 1000.0)).build_ui_state(|ui, st: &mut St| st.shell.ui(ui, &mut st.app), St { shell, app });
    h.run_steps(2);
    h.state_mut().shell.run(jcj_shell::Cmd::with(jcj_shell::commands::shell_ids::FILE_PAGE, json!({ "page": "export" })));
    h.run_steps(3);
    assert!(h.query_by_label_contains("(print PDF)").is_some());
    assert!(h.query_by_label_contains("(EPUB)").is_some());
    assert!(h.query_by_label_contains("Project bundle").is_some());
    assert!(h.query_by_label("Folder").is_some());
    h.get_by_label_contains("Project bundle").click();
    h.run_steps(2);
    assert!(h.state().app.export.bundle);
    h.get_by_label("Export Files").click();
    h.run_steps(2);
    assert!(h.state().app.export.running());
    for _ in 0..1200 {
        h.step();
        std::thread::sleep(std::time::Duration::from_millis(20));
        if !h.state().app.export.running() {
            break;
        }
    }
    h.run_steps(2);
    let last = h.state().app.export.last.clone().unwrap();
    assert!(last.starts_with("Exported 3 files"), "{last}");
}
