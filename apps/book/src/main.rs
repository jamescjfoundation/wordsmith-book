// SPDX-License-Identifier: MIT OR Apache-2.0
//! Wordsmith Book, the desktop app: the core's shell around Book's views (`wordsmith-book-ui`).

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::process::ExitCode;
use std::time::Instant;

use jcj_shell::prefs::{Prefs, PrefsFile, settings_dir};
use jcj_shell::{App as _, Shell, egui};
use wordsmith_book_ui::prefs::BookPrefs;
use wordsmith_book_ui::{BookApp, NAME, brand, shell_options};

struct Window {
    shell: Shell,
    app: BookApp,
    prefs: Option<PrefsFile<Prefs<BookPrefs>>>,
}

impl Window {
    /// Write the preferences when they changed (at most once a second unless `force`).
    fn save_prefs(&mut self, force: bool) {
        if let Some(file) = &mut self.prefs {
            let now = Prefs { shell: self.shell.prefs(), app: self.app.prefs.clone() };
            if let Err(e) = file.save(&now, Instant::now(), force) {
                self.app.message = Some(format!("Preferences could not be saved: {e}"));
            }
        }
    }
}

impl eframe::App for Window {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.shell.ui(ui, &mut self.app);
        self.save_prefs(false);
        // An installed update starts when this window closes.
        if self.shell.close_requested() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_prefs(true);
        self.shell.finish();
    }
}

/// The window's icon, from the brand's 256-pixel PNG.
fn icon() -> Option<egui::IconData> {
    let img = image::load_from_memory_with_format(brand::ICON_256, image::ImageFormat::Png).ok()?.into_rgba8();
    let (width, height) = img.dimensions();
    Some(egui::IconData { rgba: img.into_raw(), width, height })
}

fn main() -> ExitCode {
    let mut shell = Shell::new(NAME, env!("CARGO_PKG_VERSION"), brand::tokens()).with_options(shell_options()).with_updates(
        jcj_shell::updates::desktop::ReleaseSource {
            id: "wordsmith-book",
            name: NAME,
            tagline: "Make print and ebook editions of a finished book",
            repo: "jamescjfoundation/wordsmith-book",
            developer: "The James C. J. Foundation",
            version: env!("CARGO_PKG_VERSION"),
            // In place where the updater can replace the app; elsewhere the check offers the download.
            installs_in_place: !cfg!(target_os = "macos"),
        },
    );
    let mut app = BookApp::new();
    // Preferences: the shell's and Book's, from the app's settings folder.
    let prefs = settings_dir("wordsmith-book", NAME).and_then(|dir| std::fs::create_dir_all(&dir).ok().map(|()| dir)).map(|dir| {
        let (file, prefs, problem): (PrefsFile<Prefs<BookPrefs>>, _, _) = PrefsFile::open(&dir);
        shell.apply_prefs(&prefs.shell, &app.panels());
        app.prefs = prefs.app;
        app.message = problem;
        file
    });
    // The interface language (ACC-3): the preference, else the computer's.
    wordsmith_book_ui::languages::set(&app.prefs.language);
    if let Some(path) = std::env::args().nth(1)
        && let Err(e) = app.open_path(&path)
    {
        app.message = Some(e);
    }
    let window = Window { shell, app, prefs };
    let mut viewport = egui::ViewportBuilder::default().with_title(NAME).with_app_id("org.jcjfoundation.book");
    if let Some(icon) = icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions { viewport, ..eframe::NativeOptions::default() };
    match eframe::run_native(NAME, options, Box::new(|_cc| Ok(Box::new(window)))) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("could not open a window: {e}");
            ExitCode::FAILURE
        }
    }
}
