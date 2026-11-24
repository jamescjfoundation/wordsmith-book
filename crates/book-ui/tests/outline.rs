// SPDX-License-Identifier: MIT OR Apache-2.0
//! The outline (plan M1.4): model-level moves, commands with undo, the app opening the
//! Manuscript fixture's outline through the shell, and a 1,000-section outline drawn fast.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use jcj_shell::egui;
use jcj_shell::tree::{TreeState, Zone};
use jcj_shell::{App, Shell};
use serde_json::json;
use wordsmith_book_model::{BookProject, Element, ElementKind, Placement, SectionId, Source};
use wordsmith_book_ui::outline::{self, Key, READ_ONLY};
use wordsmith_book_ui::{BookApp, NAME, brand, commands, shell_options};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn tmp(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("outline").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn tree(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.insert(p.strip_prefix(dir).unwrap().display().to_string(), std::fs::read(&p).unwrap());
            }
        }
    }
    out
}

/// A Book project linking a copy of the Manuscript fixture; gives the app and the copy's folder.
fn linked_app(name: &str) -> (BookApp, PathBuf) {
    let dir = tmp(name);
    let ms = dir.join("Gulliver.wsm");
    for (rel, bytes) in tree(&root().join("tests/fixtures/novels/gullivers-travels.wsm")) {
        let p = ms.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    let mut app = BookApp::new();
    app.command(
        commands::NEW,
        &json!({ "path": dir.join("Gulliver.wsbk").display().to_string(), "manuscript": "../Gulliver.wsm", "date": "2026-11-24" }),
    )
    .unwrap();
    (app, ms)
}

fn el(id: &str, placement: Placement) -> Element {
    Element { id: id.into(), kind: Some(ElementKind::Dedication), placement, ..Element::default() }
}

#[test]
fn elements_move_among_themselves_and_never_into_the_source() {
    let mut p = BookProject::new(Source::default(), "2026-11-24");
    p.elements = vec![el("a", Placement::Front), el("b", Placement::Front), el("c", Placement::Back)];
    outline::move_element(&mut p, "a", &Key::Element("b".into()), Zone::After).unwrap();
    assert_eq!(p.elements.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(), ["b", "a", "c"]);
    outline::move_element(&mut p, "c", &Key::Element("b".into()), Zone::Before).unwrap();
    assert_eq!(
        p.elements.iter().map(|e| (e.id.as_str(), e.placement)).collect::<Vec<_>>(),
        [("c", Placement::Front), ("b", Placement::Front), ("a", Placement::Front)]
    );
    outline::move_element(&mut p, "b", &Key::Heading(Placement::Back, false), Zone::Inside).unwrap();
    assert_eq!(p.elements.last().map(|e| (e.id.as_str(), e.placement)), Some(("b", Placement::Back)));
    assert_eq!(outline::move_element(&mut p, "a", &Key::Section(SectionId::from_counter(1)), Zone::Before).unwrap_err(), READ_ONLY);
    assert!(outline::move_element(&mut p, "a", &Key::Heading(Placement::Front, true), Zone::Inside).is_err());
    assert!(outline::move_element(&mut p, "nope", &Key::Element("a".into()), Zone::Before).is_err());
}

#[test]
fn outline_commands_are_undoable_save_and_never_touch_the_manuscript() {
    let (mut app, ms) = linked_app("commands");
    let before = tree(&ms);
    let ded = app.command(commands::ADD_ELEMENT, &json!({ "kind": "dedication" })).unwrap()["id"].as_str().unwrap().to_owned();
    let about = app.command(commands::ADD_ELEMENT, &json!({ "kind": "also-by" })).unwrap()["id"].as_str().unwrap().to_owned();
    app.command(commands::SET_ELEMENT, &json!({ "id": ded, "text": "For the travellers." })).unwrap();
    app.command(commands::MOVE_ELEMENT, &json!({ "id": about, "target": { "heading": "front" }, "zone": "inside" })).unwrap();
    let ids: Vec<_> = app.book.as_ref().unwrap().files.project.elements.iter().map(|e| (e.id.clone(), e.placement)).collect();
    // The new book's own elements, then the two added; the moved one first of the front matter.
    assert_eq!(ids.first(), Some(&(about.clone(), Placement::Front)));
    assert_eq!(ids.last(), Some(&(ded.clone(), Placement::Front)));
    // A source section cannot be moved, and the error says where to change it.
    let first_chapter = app.book.as_ref().unwrap().source.as_ref().unwrap().book.body[0].children[0].id;
    let e =
        app.command(commands::MOVE_ELEMENT, &json!({ "id": ded, "target": { "section": first_chapter.as_text() }, "zone": "before" })).unwrap_err();
    assert!(e.contains("Manuscript"));
    // Undo and redo.
    app.command("edit.undo", &json!({})).unwrap();
    assert_ne!(app.book.as_ref().unwrap().files.project.elements[0].id, about);
    app.command("edit.redo", &json!({})).unwrap();
    assert_eq!(app.book.as_ref().unwrap().files.project.elements[0].id, about);
    // Save and reopen: the same elements.
    assert!(app.is_dirty());
    app.autosave().unwrap();
    assert!(!app.is_dirty());
    let path = app.book.as_ref().unwrap().root.display().to_string();
    let mut again = BookApp::new();
    again.open_path(&path).unwrap();
    assert_eq!(again.book.as_ref().unwrap().files.project.elements, app.book.as_ref().unwrap().files.project.elements);
    // The outline as data names the parts and chapters in the writer's order.
    let rows = again.command(commands::OUTLINE, &json!({})).unwrap();
    let labels: Vec<&str> = rows.as_array().unwrap().iter().filter_map(|r| r["label"].as_str()).collect();
    assert!(labels.contains(&"Part 1: A VOYAGE TO LILLIPUT") && labels.contains(&"Chapter 1"), "{labels:?}");
    assert_eq!(tree(&ms), before, "Book wrote nothing into the Manuscript project");
}

struct St {
    shell: Shell,
    app: BookApp,
}

#[test]
fn the_app_opens_the_fixtures_outline_and_shows_a_chapter() {
    let (app, _) = linked_app("window");
    let shell = Shell::new(NAME, "0.1.0", brand::tokens()).with_options(shell_options());
    let mut h =
        Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_ui_state(|ui, st: &mut St| st.shell.ui(ui, &mut st.app), St { shell, app });
    h.run();
    // Show the outline panel, as its rail button does.
    h.state_mut().shell.run(jcj_shell::Cmd::with(jcj_shell::commands::shell_ids::PANEL, json!({ "panel": "outline" })));
    h.run();
    if h.query_by_label("Part 1: A VOYAGE TO LILLIPUT").is_none() {
        h.state_mut().shell.run(jcj_shell::Cmd::with(jcj_shell::commands::shell_ids::PANEL, json!({ "panel": "outline" })));
        h.run();
    }
    assert!(h.query_by_label("Part 1: A VOYAGE TO LILLIPUT").is_some());
    h.get_by_label("Chapter 1").click();
    h.run();
    assert!(h.state().app.outline.selected.is_some());
    assert!(h.query_by_label_contains("My father had a small estate in Nottinghamshire").is_some());
}

#[test]
fn a_thousand_section_outline_draws_within_50_ms_a_frame() {
    let (mut app, _) = linked_app("thousand");
    // A book of 1,000 chapters, as the book view would give it.
    let b = &mut app.book.as_mut().unwrap().source.as_mut().unwrap().book;
    let template = b.body[0].children[0].clone();
    b.body = vec![wordsmith_manuscript_format::book::Section {
        children: (0..1000)
            .map(|i| wordsmith_manuscript_format::book::Section {
                id: SectionId::from_counter(10_000 + i),
                number: Some(i as u32 + 1),
                ..template.clone()
            })
            .collect(),
        ..b.body[0].clone()
    }];
    let project = app.book.as_ref().unwrap().files.project.clone();
    let book = app.book.as_ref().unwrap().source.as_ref().unwrap().book.clone();
    let rows = outline::rows(&project, Some(&book));
    assert!(rows.len() > 1000);
    struct T {
        state: TreeState<Key>,
        rows: Vec<jcj_shell::tree::TreeRow<Key>>,
        styled: bool,
    }
    let mut h = Harness::builder().with_size(egui::vec2(320.0, 800.0)).build_ui_state(
        |ui, t: &mut T| {
            if !t.styled {
                let tokens = brand::tokens();
                jcj_shell::theme::install_fonts(ui.ctx(), &tokens.fonts, &jcj_shell::theme::default_faces());
                jcj_shell::theme::apply(ui.ctx(), &tokens.theme(jcj_shell::theme::Mode::Light));
                t.styled = true;
            }
            let _ = jcj_shell::tree::tree(ui, "o", &mut t.state, &t.rows);
        },
        T { state: TreeState::default(), rows, styled: false },
    );
    h.run();
    let start = std::time::Instant::now();
    for _ in 0..20 {
        h.step();
    }
    let per_frame = start.elapsed() / 20;
    assert!(per_frame.as_millis() < 50, "{per_frame:?} a frame");
}
