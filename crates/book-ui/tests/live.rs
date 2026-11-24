// SPDX-License-Identifier: MIT OR Apache-2.0
//! Live previews (plan M7.3): a design change reaches both previews, and on the 150,000-word
//! test book (Waverley, about 210,000 words) the print preview's first page shows within a
//! second of the change (docs/performance.md).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use egui_kittest::Harness;
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

fn gulliver_app() -> BookApp {
    let dir = tmp("live-gulliver");
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

/// Bring both previews up to date with the book.
fn refresh(app: &mut BookApp) {
    let b = app.book.as_ref().unwrap();
    for _ in 0..2 {
        app.print_preview.update(b);
        assert!(app.print_preview.wait(Duration::from_secs(120)));
        app.ebook_preview.update(b);
        assert!(app.ebook_preview.wait(Duration::from_secs(60)));
    }
}

#[test]
fn a_design_change_reaches_both_previews() {
    let mut app = gulliver_app();
    refresh(&mut app);
    let before_rev = app.book.as_ref().unwrap().revision;
    let print_before = app.print_preview.built.clone().unwrap();
    let ebook_before = app.ebook_preview.shown_pixels().unwrap();
    assert_eq!(print_before.revision, before_rev);
    app.command(commands::DESIGN_CHOOSE, &json!({ "id": "thriller" })).unwrap();
    let rev = app.book.as_ref().unwrap().revision;
    assert_ne!(rev, before_rev);
    refresh(&mut app);
    let print_after = app.print_preview.built.clone().unwrap();
    assert_eq!(print_after.revision, rev);
    assert!(!print_after.opening);
    assert_ne!(print_after.print.look.body_font, print_before.print.look.body_font, "the print preview uses the new design");
    assert_eq!(app.ebook_preview.laid.as_ref().unwrap().key.0, rev);
    assert_ne!(app.ebook_preview.shown_pixels().unwrap().2, ebook_before.2, "the ebook preview shows the new design");
}

struct St {
    shell: Shell,
    app: BookApp,
}

#[test]
fn the_first_preview_page_shows_within_a_second_of_a_design_change_on_the_long_book() {
    let dir = tmp("live-waverley");
    let mut app = BookApp::new();
    app.command(
        commands::IMPORT,
        &json!({ "file": root().join("tests/fixtures/novels/waverley.docx").display().to_string(), "path": dir.join("Waverley.wsbk").display().to_string() }),
    )
    .unwrap();
    let shell = Shell::new(NAME, "0.1.0", brand::tokens()).with_options(shell_options());
    let mut h =
        Harness::builder().with_size(egui::vec2(1600.0, 1000.0)).build_ui_state(|ui, st: &mut St| st.shell.ui(ui, &mut st.app), St { shell, app });
    h.run_steps(2);
    h.state_mut().shell.run(jcj_shell::Cmd::with(jcj_shell::commands::shell_ids::PANEL, json!({ "panel": "preview" })));
    let wait_for_page = |h: &mut Harness<'_, St>, limit: Duration| -> Duration {
        let start = Instant::now();
        while start.elapsed() < limit {
            h.step();
            let rev = h.state().app.book.as_ref().unwrap().revision;
            let p = &h.state().app.print_preview;
            if p.built.as_ref().is_some_and(|b| b.revision == rev) && p.page_ready(0) {
                return start.elapsed();
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        panic!("no first page within {limit:?}");
    };
    let first_open = wait_for_page(&mut h, Duration::from_secs(60));
    // Let the whole book finish, as a writer would before changing the design.
    let start = Instant::now();
    while h.state().app.print_preview.building() && start.elapsed() < Duration::from_secs(120) {
        h.step();
        std::thread::sleep(Duration::from_millis(5));
    }
    let whole = h.state().app.print_preview.built.as_ref().unwrap().took;
    let pages = h.state().app.print_preview.built.as_ref().unwrap().print.typeset.page_count();
    let mut after_change = Vec::new();
    for design in ["literary", "fantasy", "classic"] {
        h.state_mut().app.command(commands::DESIGN_CHOOSE, &json!({ "id": design })).unwrap();
        after_change.push(wait_for_page(&mut h, Duration::from_secs(30)));
        while h.state().app.print_preview.building() {
            h.step();
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    eprintln!(
        "timing: first page on opening {first_open:?}; whole book ({pages} pages) {whole:?}; first page after a design change {after_change:?}"
    );
    let worst = after_change.iter().max().copied().unwrap();
    assert!(worst < Duration::from_secs(1), "the first page took {worst:?} after a design change");
}
