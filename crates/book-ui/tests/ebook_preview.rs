// SPDX-License-Identifier: MIT OR Apache-2.0
//! The ebook preview (plan M7.2, EBK-8): each device lays out the fixture's first chapter; text
//! size changes the layout and dark page the colors; every control is named in the harness.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use jcj_shell::{Shell, egui};
use serde_json::json;
use wordsmith_book_typeset::screen::{DEVICES, TEXT_SIZES};
use wordsmith_book_ui::ebook_preview::{EbookPreview, brightness, darken};
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

fn laid(p: &mut EbookPreview, app: &BookApp) -> usize {
    let b = app.book.as_ref().unwrap();
    p.update(b);
    assert!(p.wait(Duration::from_secs(60)));
    assert!(p.error.is_none(), "{:?}", p.error);
    p.laid.as_ref().unwrap().pages.len()
}

#[test]
fn each_device_lays_out_the_first_chapter_and_text_size_and_dark_page_change_it() {
    let app = linked_app("ebook-preview");
    let mut p = EbookPreview::default();
    let mut sizes = Vec::new();
    for d in DEVICES {
        p.device = d.id;
        let pages = laid(&mut p, &app);
        assert!(pages >= 2, "{}: {pages} pages", d.id);
        let (w, h, _) = p.shown_pixels().unwrap();
        assert!(h > w, "{}: a portrait screen", d.id);
        sizes.push((w, h, pages));
    }
    // Bigger screens hold more of the chapter on a page.
    assert!(sizes[0].2 > sizes[2].2, "{sizes:?}");
    assert!(sizes[0].0 < sizes[2].0);
    // Larger text, more pages.
    p.device = "phone";
    p.text = 0;
    let small = laid(&mut p, &app);
    p.text = TEXT_SIZES.len() - 1;
    let large = laid(&mut p, &app);
    assert!(large > small, "small text {small} pages, extra large {large}");
    // Dark page: the same page, its colors turned.
    let light = p.shown_pixels().unwrap().2;
    p.dark = true;
    let dark = p.shown_pixels().unwrap().2;
    assert!(brightness(&light) > 200.0 && brightness(&dark) < 80.0, "{} {}", brightness(&light), brightness(&dark));
    assert_eq!(darken(&[255, 255, 255, 255, 0, 0, 0, 255])[..3], [24, 24, 24]);
    // Open in a reading app writes the ebook (here without starting an app).
    let mut app = app;
    let v = app.command(commands::EBOOK_OPEN, &json!({ "open": false })).unwrap();
    let path = PathBuf::from(v["path"].as_str().unwrap());
    assert!(std::fs::read(&path).unwrap().starts_with(b"PK"));
}

struct St {
    shell: Shell,
    app: BookApp,
}

#[test]
fn every_control_is_named_in_the_harness() {
    let app = linked_app("ebook-preview-ui");
    let shell = Shell::new(NAME, "0.1.0", brand::BRAND.tokens()).with_options(shell_options());
    let mut h =
        Harness::builder().with_size(egui::vec2(1600.0, 1000.0)).build_ui_state(|ui, st: &mut St| st.shell.ui(ui, &mut st.app), St { shell, app });
    h.run_steps(2);
    h.state_mut().shell.run(jcj_shell::Cmd::with(jcj_shell::commands::shell_ids::PANEL, json!({ "panel": "ebook" })));
    for _ in 0..400 {
        h.step();
        std::thread::sleep(Duration::from_millis(10));
        if h.query_by_label("Next Page").is_some() {
            break;
        }
    }
    assert!(h.query_by_label_contains("Preview: reading apps").is_some());
    for name in DEVICES.iter().map(|d| d.name).chain(TEXT_SIZES.iter().map(|t| t.0)) {
        assert!(h.query_by_label(name).is_some(), "{name}");
    }
    for name in ["Dark page", "Open in a Reading App", "Previous Page", "Next Page"] {
        assert!(h.query_by_label(name).is_some(), "{name}");
    }
    assert!(h.query_by_label_contains("Page 1 of").is_some());
    h.get_by_label("Next Page").click();
    h.run_steps(2);
    assert_eq!(h.state().app.ebook_preview.page, 1);
    h.get_by_label("Tablet").click();
    h.run_steps(2);
    assert_eq!(h.state().app.ebook_preview.device, "tablet");
    h.get_by_label("Dark page").click();
    h.run_steps(2);
    assert!(h.state().app.ebook_preview.dark);
}
