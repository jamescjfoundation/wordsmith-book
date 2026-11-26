// SPDX-License-Identifier: MIT OR Apache-2.0
//! The import review (IMP-4, plan section 8): the file's paragraphs beside the structure found in
//! them, the detection rules to switch on and off, and the import itself.

use std::path::{Path, PathBuf};

use jcj_shell::egui;
use jcj_shell::i18n::{tr, tr_with};
use serde_json::{Value, json};
use wordsmith_book_import::detect::{self, Kind, Mark, Para};
use wordsmith_book_import::{Format, Report};
use wordsmith_book_model::{ImportRules, Source};

use crate::doc::OpenBook;

/// The most bytes read from a file to import.
pub const MAX_FILE: u64 = 256 << 20;

/// A file being reviewed before import.
pub struct Review {
    /// The file.
    pub file: PathBuf,
    /// Where the new book goes.
    pub target: String,
    format: Format,
    doc: jcj_doc::Document,
    paras: Vec<Para>,
    /// The rules in force.
    pub rules: ImportRules,
    /// What each paragraph is under the rules.
    pub marks: Vec<Mark>,
}

/// The format of a file, by its extension.
pub fn format_of(path: &Path) -> Result<Format, String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    Format::from_extension(&ext).filter(|f| *f != Format::Html).ok_or_else(|| format!("Book cannot import `.{ext}` files"))
}

fn read_file(path: &Path) -> Result<Vec<u8>, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if meta.len() > MAX_FILE {
        return Err(format!("{} is larger than {} MB", path.display(), MAX_FILE >> 20));
    }
    std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
}

impl Review {
    /// Read `file` and find its structure with the default rules.
    pub fn open(file: &Path) -> Result<Review, String> {
        let format = format_of(file)?;
        let bytes = read_file(file)?;
        let rules = ImportRules::default();
        let doc = if format == Format::Text {
            wordsmith_book_import::text::read(std::str::from_utf8(&bytes).map_err(|_| "the text is not UTF-8".to_owned())?, &rules)
        } else {
            wordsmith_book_import::read(format, &bytes).map_err(|e| e.to_string())?
        };
        let paras = wordsmith_book_import::paras_of(&doc);
        let marks = detect::detect(&paras, &rules);
        let target = file.with_extension(wordsmith_book_format::EXTENSION).display().to_string();
        Ok(Review { file: file.to_path_buf(), target, format, doc, paras, rules, marks })
    }

    /// Switch a rule on or off and find the structure again.
    pub fn set_rule(&mut self, name: &str, on: bool) {
        self.rules.off.retain(|r| r != name);
        if !on {
            self.rules.off.push(name.to_owned());
        }
        self.marks = detect::detect(&self.paras, &self.rules);
    }

    /// How many of each kind of section the rules find.
    pub fn counts(&self) -> (usize, usize, usize) {
        let n = |k: Kind| self.marks.iter().filter(|m| matches!(m, Mark::Heading(x, _) if *x == k)).count();
        (n(Kind::Part), n(Kind::Chapter), self.marks.iter().filter(|m| matches!(m, Mark::Break(_))).count())
    }

    /// Import: write the owned source and create the book at the target.
    pub fn import(&self) -> Result<(OpenBook, Report), String> {
        let title = self.file.file_stem().map_or_else(|| "Untitled".into(), |n| n.to_string_lossy().into_owned());
        let imported = wordsmith_book_import::import_document(&self.doc, &self.rules, &title).map_err(|e| e.to_string())?;
        let target = PathBuf::from(&self.target);
        if target.join("book.toml").exists() {
            return Err(format!("{} already holds a book", target.display()));
        }
        std::fs::create_dir_all(&target).map_err(|e| e.to_string())?;
        wordsmith_book_format::source::write_owned(&target, wordsmith_book_model::OWNED_SOURCE, &imported.project).map_err(|e| e.to_string())?;
        let mut book = OpenBook::create(&target, Source::default(), &crate::today())?;
        let rules = self.rules.clone();
        book.edit(|f| {
            f.project.cleanup.rules = rules;
            f.project.elements = wordsmith_book_model::matter::default_elements();
            Ok(())
        })?;
        book.save()?;
        let _ = self.format;
        Ok((book, imported.report))
    }

    /// Draw the review; true when the user asked to import.
    pub fn ui(&mut self, ui: &mut egui::Ui) -> (bool, bool) {
        let (parts, chapters, breaks) = self.counts();
        let (p, c, b) = (parts.to_string(), chapters.to_string(), breaks.to_string());
        let file = self.file.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        ui.label(tr_with(
            "{file}: {parts} parts, {chapters} chapters, {breaks} scene breaks found.",
            &[("file", &file), ("parts", &p), ("chapters", &c), ("breaks", &b)],
        ));
        ui.separator();
        ui.label(tr("Rules"));
        let mut changes: Vec<(String, bool)> = Vec::new();
        for (name, about) in detect::RULES {
            let mut on = !self.rules.off.iter().any(|r| r == name);
            if ui.checkbox(&mut on, about).changed() {
                changes.push((name.to_owned(), on));
            }
        }
        for (name, on) in changes {
            self.set_rule(&name, on);
        }
        ui.separator();
        ui.horizontal(|ui| {
            let label = ui.label(tr("New book:"));
            ui.text_edit_singleline(&mut self.target).labelled_by(label.id);
        });
        let mut go = false;
        let mut cancel = false;
        ui.horizontal(|ui| {
            go = ui.button(tr("Import")).clicked();
            cancel = ui.button(tr("Cancel")).clicked();
        });
        ui.separator();
        egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
            egui::Grid::new("import-review").striped(true).show(ui, |ui| {
                for (p, m) in self.paras.iter().zip(&self.marks).take(2000) {
                    let tag = match m {
                        Mark::Heading(Kind::Part, _) => tr("Part"),
                        Mark::Heading(Kind::Chapter, _) => tr("Chapter"),
                        Mark::Heading(Kind::Front, _) => tr("Front matter"),
                        Mark::Heading(Kind::Back, _) => tr("Back matter"),
                        Mark::Break(_) => tr("Scene break"),
                        Mark::Skip => tr("(left out)"),
                        Mark::Body => String::new(),
                    };
                    ui.label(tag);
                    ui.label(p.text.chars().take(90).collect::<String>());
                    ui.end_row();
                }
            });
        });
        (go, cancel)
    }
}

/// A report as JSON, for agents and the command line.
pub fn report_json(r: &Report) -> Value {
    json!({
        "made": r.made,
        "kept": r.kept,
        "dropped": r.dropped,
        "changes": r.changes,
        "words": r.words.as_ref().map(ToString::to_string),
        "notes": r.notes.as_ref().map(ToString::to_string),
        "passed": r.passed(),
    })
}
