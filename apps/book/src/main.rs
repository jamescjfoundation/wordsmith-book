// SPDX-License-Identifier: MIT OR Apache-2.0
//! Wordsmith Book, the desktop app: the core's shell around Book's views (`wordsmith-book-ui`).

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::process::ExitCode;

use jcj_shell::Shell;
use jcj_shell::egui;
use wordsmith_book_ui::{BookApp, NAME, brand, shell_options};

struct Window {
    shell: Shell,
    app: BookApp,
}

impl eframe::App for Window {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.shell.ui(ui, &mut self.app);
    }
}

/// The window's icon, from the brand's 256-pixel PNG.
fn icon() -> Option<egui::IconData> {
    let img = image::load_from_memory_with_format(brand::ICON_256, image::ImageFormat::Png).ok()?.into_rgba8();
    let (width, height) = img.dimensions();
    Some(egui::IconData { rgba: img.into_raw(), width, height })
}

fn main() -> ExitCode {
    let shell = Shell::new(NAME, env!("CARGO_PKG_VERSION"), brand::tokens()).with_options(shell_options());
    let window = Window { shell, app: BookApp::new() };
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
