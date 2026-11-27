// SPDX-License-Identifier: MIT OR Apache-2.0
//! The README's pictures, drawn by the app itself offscreen (`jcj_shell::snapshot`):
//! `cargo xtask screenshots` runs this with the folder to write them to.
//!
//! The sample book is *Gulliver's Travels* (1726, public domain), linked from the Manuscript
//! fixture `tests/fixtures/novels/gullivers-travels.wsm`, which is copied to a temporary folder
//! first so nothing in the repository is touched.

use std::path::{Path, PathBuf};
use std::time::Duration;

use jcj_shell::egui::{self, Vec2};
use jcj_shell::theme::Mode;
use jcj_shell::workspace::Layout;
use jcj_shell::{App as _, Shell};
use serde_json::json;
use wordsmith_book_ui::outline::Key;
use wordsmith_book_ui::{BookApp, NAME, brand, commands, shell_options};
use wordsmith_manuscript_format::book::Section;
use wordsmith_manuscript_model::Role;

const SIZE: Vec2 = Vec2::new(1280.0, 820.0);
const PPP: f32 = 1.5;

fn main() {
    let out = std::env::args().nth(1).map_or_else(|| PathBuf::from("docs/images"), PathBuf::from);
    if let Err(e) = run(&out) {
        eprintln!("screenshots: {e}");
        std::process::exit(1);
    }
}

fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    for entry in std::fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        let Some(name) = path.file_name() else { continue };
        if path.is_dir() {
            copy_tree(&path, &to.join(name))?;
        } else {
            std::fs::copy(&path, to.join(name)).map_err(|e| format!("{}: {e}", path.display()))?;
        }
    }
    Ok(())
}

fn first_chapter(sections: &[Section]) -> Option<&Section> {
    sections.iter().find_map(|s| if s.role == Role::Chapter { Some(s) } else { first_chapter(&s.children) })
}

/// A Book project in a temporary folder, linked to a copy of the Gulliver fixture, with its
/// first chapter selected.
fn sample(dir: &Path) -> Result<BookApp, String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let _ = std::fs::remove_dir_all(dir);
    copy_tree(&root.join("tests/fixtures/novels/gullivers-travels.wsm"), &dir.join("Gulliver.wsm"))?;
    let mut app = BookApp::new();
    let path = dir.join("Gulliver.wsbk");
    app.command(commands::NEW, &json!({ "path": path.display().to_string(), "manuscript": "../Gulliver.wsm", "date": "2026-11-25" }))?;
    let chapter = app
        .book
        .as_ref()
        .and_then(|b| b.source.as_ref().ok())
        .and_then(|s| first_chapter(&s.book.body))
        .map(|s| s.id)
        .ok_or("the sample has no chapter")?;
    app.outline.selected = Some(Key::Section(chapter));
    Ok(app)
}

/// One picture, with the given panels open, in the given theme. Each picture has its own
/// context and shell (the shell installs its fonts and style on its first frame), so the
/// previews' page images are made again for it; the frames are spaced out so the background
/// builds can finish before the last one.
fn shot(app: &mut BookApp, out: &Path, name: &str, open: &[&str], mode: Mode, frames: usize) -> Result<(), String> {
    let mut shell = Shell::new(NAME, env!("CARGO_PKG_VERSION"), brand::BRAND.tokens()).with_options(shell_options());
    shell.tour_seen = true;
    shell.mode = mode;
    shell.layout = Layout::with_open(&app.panels(), open);
    app.print_preview = Default::default();
    let (device, dark, text) = (app.ebook_preview.device, app.ebook_preview.dark, app.ebook_preview.text);
    app.ebook_preview = Default::default();
    (app.ebook_preview.device, app.ebook_preview.dark, app.ebook_preview.text) = (device, dark, text);
    let ctx = egui::Context::default();
    let pic = jcj_shell::snapshot::render(&ctx, SIZE, PPP, Vec::new(), frames, |ui| {
        shell.ui(ui, app);
        if frames > 6 {
            std::thread::sleep(Duration::from_millis(20));
        }
    });
    let path = out.join(format!("{name}.png"));
    std::fs::write(&path, pic.to_png()).map_err(|e| format!("{}: {e}", path.display()))?;
    println!("{}", path.display());
    Ok(())
}

fn run(out: &Path) -> Result<(), String> {
    std::fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let work = std::env::temp_dir().join("wordsmith-book-screenshots");
    let a = &mut sample(&work)?;
    // The chapter beside its print pages.
    shot(a, out, "print-preview", &["outline", "preview"], Mode::Light, 250)?;
    // The design chooser.
    shot(a, out, "designs", &["outline", "designs"], Mode::Light, 8)?;
    // The ebook as a phone shows it.
    shot(a, out, "ebook", &["outline", "ebook"], Mode::Light, 250)?;
    // Details: metadata, what is still needed, editions.
    shot(a, out, "details", &["outline", "details"], Mode::Light, 8)?;
    // Preflight, run on the paperback.
    let validator = a.prefs.validator_settings();
    if let Some(b) = a.book.as_ref() {
        a.preflight.start(b, "paperback", validator);
    }
    if !a.preflight.wait(Duration::from_secs(300)) {
        return Err("preflight did not finish".into());
    }
    shot(a, out, "preflight", &["outline", "preflight"], Mode::Light, 8)?;
    // Dark: the ebook on a tablet with a dark page.
    a.ebook_preview.device = "tablet";
    a.ebook_preview.dark = true;
    shot(a, out, "ebook-dark", &["outline", "ebook"], Mode::Dark, 250)?;
    let _ = std::fs::remove_dir_all(&work);
    Ok(())
}
