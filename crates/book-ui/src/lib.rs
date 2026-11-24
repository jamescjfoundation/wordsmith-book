// SPDX-License-Identifier: MIT OR Apache-2.0
//! Wordsmith Book's views: outline, text, design chooser, settings, previews, preflight, export,
//! ribbon tabs, panels and agent tools. The window itself is the core's shell (`jcj-shell`); this
//! crate supplies what is about making a book.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod brand;

use jcj_shell::egui;
use jcj_shell::{About, App, Cmd, CommandInfo, Commands, Shell, ShellOptions};

/// The app's name as shown in the window and the About page.
pub const NAME: &str = "Wordsmith Book";

/// The app's id in project files and preferences (core decision 0006).
pub const APP_ID: &str = "org.jcjfoundation.book";

/// The shell's options for Book's window: the start window shows while no book is open.
pub fn shell_options() -> ShellOptions {
    ShellOptions { start_window: true, ..ShellOptions::default() }
}

/// The desktop app: the shell's window around Book's views.
#[derive(Default)]
pub struct BookApp {
    revision: u64,
}

impl BookApp {
    /// An app with no book open.
    pub fn new() -> BookApp {
        BookApp::default()
    }
}

impl Commands for BookApp {
    fn command(&self, id: &str) -> Option<CommandInfo> {
        self.all_commands().into_iter().find(|c| c.id == id)
    }

    fn all_commands(&self) -> Vec<CommandInfo> {
        Vec::new()
    }
}

impl App for BookApp {
    fn document_name(&self) -> String {
        "No book open".into()
    }

    fn revision(&self) -> u64 {
        self.revision
    }

    fn run(&mut self, cmd: &Cmd, _shell: &mut Shell) -> Result<(), String> {
        Err(format!("unknown command {}", cmd.id))
    }

    fn document_ui(&mut self, ui: &mut egui::Ui, _shell: &mut Shell) {
        ui.centered_and_justified(|ui| ui.label("Open or create a book to begin."));
    }

    fn mark_width(&self, height: f32) -> f32 {
        brand::mark_width(height)
    }

    fn paint_mark(&self, painter: &egui::Painter, rect: egui::Rect, ink: egui::Color32) {
        brand::paint_mark(painter, rect, ink);
    }

    fn has_document(&self) -> bool {
        false
    }

    fn about(&self) -> About {
        About {
            summary: "Book design and typesetting: print-ready PDFs and accessible EPUBs from a finished manuscript.".into(),
            credit: "Built on open-source-core, based on WordCraft by the ArtCraft team.".into(),
            acknowledgments: String::new(),
            website: Some("https://github.com/jamescjfoundation/wordsmith-book".into()),
        }
    }
}
