// SPDX-License-Identifier: MIT OR Apache-2.0
//! Book's own preferences, kept with the shell's in the app's preferences file: where the
//! standard EPUB validator is (PRE-2), the Java runtime that runs it, where the accessibility
//! checker is, their time limit, and the interface language.

use jcj_shell::egui;
use jcj_shell::i18n::{tl, tr};
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
    /// The interface language's code (ACC-3; empty: the computer's).
    pub language: String,
    /// The PDF validator: a program (empty: not set up; PRT-12).
    pub pdf_validator: String,
    /// The publishing house's file, `house.toml` (empty: none; DSN-7, META-3 to META-5).
    pub house_file: String,
}

impl BookPrefs {
    /// The validator's settings.
    pub fn validator_settings(&self) -> Settings {
        Settings {
            program: self.validator.clone(),
            java: self.java.clone(),
            seconds: self.validator_seconds.min(3600),
            access: self.access_checker.clone(),
            pdf: self.pdf_validator.clone(),
        }
    }
}

/// The preferences, as the Preferences page draws them.
pub fn ui(ui: &mut egui::Ui, p: &mut BookPrefs) {
    ui.heading(tr("Language"));
    ui.horizontal(|ui| {
        let mut chosen = None;
        if ui.selectable_label(p.language.is_empty(), tr("The computer's")).clicked() {
            chosen = Some(String::new());
        }
        for (code, name) in crate::languages::LANGUAGES {
            if ui.selectable_label(p.language == code, name).clicked() {
                chosen = Some(code.to_owned());
            }
        }
        if let Some(code) = chosen.filter(|c| *c != p.language) {
            p.language = code;
            crate::languages::set(&p.language);
        }
    });
    ui.separator();
    ui.heading(tr("Checking ebooks"));
    ui.label(tr("Preflight runs the standard EPUB validator and an accessibility checker too when you have installed them and say where they are."));
    let field = |ui: &mut egui::Ui, name: &str, value: &mut String, hint: &str| {
        ui.horizontal(|ui| {
            let label = ui.label(tr(name));
            ui.add(egui::TextEdit::singleline(value).hint_text(tr(hint)).desired_width(360.0)).labelled_by(label.id);
        });
    };
    field(ui, tl("EPUB validator"), &mut p.validator, tl("its .jar file, or a program"));
    field(ui, tl("Java runtime"), &mut p.java, tl("leave empty to use java"));
    field(ui, tl("Accessibility checker"), &mut p.access_checker, tl("its program"));
    field(ui, tl("PDF validator"), &mut p.pdf_validator, tl("its program, for PDF/A and PDF/UA"));
    ui.separator();
    ui.heading(tr("Publishing house"));
    field(ui, tl("House file"), &mut p.house_file, tl("its house.toml: imprints, series and ISBNs"));
    ui.horizontal(|ui| {
        let label = ui.label(tr("Time limit (seconds)"));
        ui.add(egui::DragValue::new(&mut p.validator_seconds).range(0..=3600)).labelled_by(label.id);
    });
}
