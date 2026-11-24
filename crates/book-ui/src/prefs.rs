// SPDX-License-Identifier: MIT OR Apache-2.0
//! Book's own preferences, kept with the shell's in the app's preferences file: where the
//! standard EPUB validator is (PRE-2), the Java runtime that runs it, and its time limit.

use jcj_shell::egui;
use serde::{Deserialize, Serialize};
use wordsmith_book_check::validator::Settings;

/// Book's preferences.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct BookPrefs {
    /// The EPUB validator: its `.jar` or a program (empty: not set up).
    pub validator: String,
    /// The Java runtime for a `.jar` (empty: `java` on the path).
    pub java: String,
    /// The validator's time limit in seconds (0: two minutes).
    pub validator_seconds: u32,
}

impl BookPrefs {
    /// The validator's settings.
    pub fn validator_settings(&self) -> Settings {
        Settings { program: self.validator.clone(), java: self.java.clone(), seconds: self.validator_seconds.min(3600) }
    }
}

/// The preferences, as the Preferences page draws them.
pub fn ui(ui: &mut egui::Ui, p: &mut BookPrefs) {
    ui.heading("Checking ebooks");
    ui.label("Preflight runs the standard EPUB validator too when you have installed it and say where it is.");
    let field = |ui: &mut egui::Ui, name: &str, value: &mut String, hint: &str| {
        ui.horizontal(|ui| {
            let label = ui.label(name);
            ui.add(egui::TextEdit::singleline(value).hint_text(hint).desired_width(360.0)).labelled_by(label.id);
        });
    };
    field(ui, "EPUB validator", &mut p.validator, "its .jar file, or a program");
    field(ui, "Java runtime", &mut p.java, "leave empty to use java");
    ui.horizontal(|ui| {
        let label = ui.label("Time limit (seconds)");
        ui.add(egui::DragValue::new(&mut p.validator_seconds).range(0..=3600)).labelled_by(label.id);
    });
}
