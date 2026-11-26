// SPDX-License-Identifier: MIT OR Apache-2.0
//! The print preview (plan M7.1, PRV-1): spreads, a page click opens its section and "Show in
//! Print Preview" comes back to it, and frames stay quick while a build runs.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use jcj_shell::{Shell, egui};
use serde_json::json;
use wordsmith_book_ui::outline::{Key, key_of_piece};
use wordsmith_book_ui::preview::{spread_of, spreads};
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

/// A Book project linking a copy of the Manuscript fixture.
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
fn spreads_put_the_first_page_alone_on_the_right() {
    assert!(spreads(0).is_empty());
    assert_eq!(spreads(1), vec![(None, Some(0))]);
    assert_eq!(spreads(4), vec![(None, Some(0)), (Some(1), Some(2)), (Some(3), None)]);
    assert_eq!(spreads(5).len(), 3);
    assert_eq!((spread_of(0), spread_of(1), spread_of(2), spread_of(3)), (0, 1, 1, 2));
}

#[test]
fn the_preview_builds_in_the_background_and_pages_open_their_sections() {
    let app = linked_app("preview");
    let shell = Shell::new(NAME, "0.1.0", brand::tokens()).with_options(shell_options());
    let mut h =
        Harness::builder().with_size(egui::vec2(1600.0, 1000.0)).build_ui_state(|ui, st: &mut St| st.shell.ui(ui, &mut st.app), St { shell, app });
    h.run_steps(2);
    h.state_mut().shell.run(jcj_shell::Cmd::with(jcj_shell::commands::shell_ids::PANEL, json!({ "panel": "preview" })));
    // Frames while the build runs: the interface never waits for it.
    let mut slowest = Duration::ZERO;
    let mut times = Vec::new();
    let mut frames = 0;
    // How long this machine holds a thread up, measured where none of Book's code runs: by how
    // much the short pause between two frames overruns.
    let pause = Duration::from_millis(5);
    let mut worst_overrun = Duration::ZERO;
    let start = Instant::now();
    for _ in 0..5 {
        h.step();
        if h.state().app.print_preview.building() {
            break;
        }
    }
    assert!(
        h.state().app.print_preview.building(),
        "opening the preview starts a build: {:?}, open: {}",
        h.state().app.print_preview.error,
        h.state().shell.layout.is_open("preview")
    );
    while h.state().app.print_preview.building() && start.elapsed() < Duration::from_secs(120) {
        let t = Instant::now();
        h.step();
        let took = t.elapsed();
        if frames > 0 {
            slowest = slowest.max(took);
            times.push(took);
        }
        frames += 1;
        let paused = Instant::now();
        std::thread::sleep(pause);
        worst_overrun = worst_overrun.max(paused.elapsed().saturating_sub(pause));
    }
    assert!(!h.state().app.print_preview.building(), "the build finished");
    assert!(frames > 3, "frames ran while the book was laid out ({frames})");
    // Frames stay under 16 ms while the build runs (the median; on a loaded machine one frame
    // can cross it, but none ever waits on the build, which would take far longer).
    times.sort();
    let median = times[times.len() / 2];
    assert!(median < Duration::from_millis(16), "the median frame took {median:?} while the build ran");
    // No frame waits on the build: 100 ms at most, where the build takes seconds. A busy
    // machine holds every thread up now and then, which says nothing about Book, so a frame
    // is also given what the machine took from the pauses (a frame works for a few
    // milliseconds and is held up more often than a pause that only has to wake: four of the
    // worst). With nothing else running the pauses overrun by next to nothing, and the limit
    // is the 100 ms.
    let limit = Duration::from_millis(100) + 4 * worst_overrun;
    assert!(slowest < limit, "a frame took {slowest:?} while the build ran (limit {limit:?}; a {pause:?} pause overran by {worst_overrun:?})");
    // Page images arrive from the drawing thread.
    for _ in 0..200 {
        h.step();
        std::thread::sleep(Duration::from_millis(10));
        if h.query_by_label_contains("pages · gutter").is_some() && h.query_by_label("Page 1").is_some() {
            break;
        }
    }
    assert!(h.query_by_label("Page 1").is_some(), "the first spread shows");
    // A page click opens its section.
    let built = h.state().app.print_preview.built.clone().unwrap();
    let piece = built.print.typeset.section_of_page(2).unwrap();
    h.get_by_label("Page 3").click();
    h.run_steps(2);
    assert_eq!(h.state().app.outline.selected, Some(key_of_piece(&piece)));
    // And back: a chapter far into the book, shown in the preview, scrolls its page into view.
    let map = built.print.typeset.page_map();
    let far = map.iter().rev().find(|s| !s.id.starts_with("element:")).unwrap().clone();
    assert!(h.query_by_label(&format!("Page {}", far.page)).is_none(), "the page is not in view yet");
    h.state_mut().app.outline.selected = Some(key_of_piece(&far.id));
    assert!(matches!(h.state().app.outline.selected, Some(Key::Section(_))));
    h.run_steps(2);
    h.get_by_label("Show in Print Preview").click();
    h.run_steps(4);
    assert!(h.query_by_label(&format!("Page {}", far.page)).is_some(), "page {} is in view", far.page);
    // The same through commands, as agents use them.
    let page = h.state_mut().app.command(commands::PREVIEW_SHOW, &json!({ "section": far.id })).unwrap();
    assert_eq!(page["page"], far.page);
    let back = h.state_mut().app.command(commands::PREVIEW_PAGE, &json!({ "page": far.page })).unwrap();
    assert_eq!(back["section"], far.id.as_str());
    assert!(h.state_mut().app.command(commands::PREVIEW_PAGE, &json!({ "page": 100_000 })).is_err());
}
