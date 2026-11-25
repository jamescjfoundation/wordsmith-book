// SPDX-License-Identifier: MIT OR Apache-2.0
//! Book's own preferences, kept with the shell's in the app's preferences file: where the
//! standard EPUB validator is (PRE-2), the Java runtime that runs it, where the accessibility
//! checker is, and their time limit.

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
    /// The time limit in seconds for each checking program (0: two minutes).
    pub validator_seconds: u32,
    /// The accessibility checker: a program (empty: not set up).
    pub access_checker: String,
}

impl BookPrefs {
    /// The validator's settings.
    pub fn validator_settings(&self) -> Settings {
        Settings {
            program: self.validator.clone(),
            java: self.java.clone(),
            seconds: self.validator_seconds.min(3600),
            access: self.access_checker.clone(),
        }
    }
}

/// The preferences, as the Preferences page draws them.
pub fn ui(ui: &mut egui::Ui, p: &mut BookPrefs) {
    ui.heading("Checking ebooks");
    ui.label("Preflight runs the standard EPUB validator and an accessibility checker too when you have installed them and say where they are.");
    let field = |ui: &mut egui::Ui, name: &str, value: &mut String, hint: &str| {
        ui.horizontal(|ui| {
            let label = ui.label(name);
            ui.add(egui::TextEdit::singleline(value).hint_text(hint).desired_width(360.0)).labelled_by(label.id);
        });
    };
    field(ui, "EPUB validator", &mut p.validator, "its .jar file, or a program");
    field(ui, "Java runtime", &mut p.java, "leave empty to use java");
    field(ui, "Accessibility checker", &mut p.access_checker, "its program");
    ui.horizontal(|ui| {
        let label = ui.label("Time limit (seconds)");
        ui.add(egui::DragValue::new(&mut p.validator_seconds).range(0..=3600)).labelled_by(label.id);
    });
}
