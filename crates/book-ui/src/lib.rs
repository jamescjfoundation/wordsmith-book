// SPDX-License-Identifier: MIT OR Apache-2.0
//! Wordsmith Book's views: outline, text, design chooser, settings, previews, preflight, export,
//! ribbon tabs, panels and agent tools. The window itself is the core's shell (`jcj-shell`); this
//! crate supplies what is about making a book.
//!
//! Every change is a command ([`BookApp::command`]) with an id and JSON parameters: the ribbon,
//! the outline, keys, agents and tests all go through it, and each change is one undo step.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod brand;
pub mod chooser;
pub mod cleanup;
pub mod commands;
pub mod designs;
pub mod doc;
pub mod import_review;
pub mod metadata;
pub mod outline;

use std::path::PathBuf;

use jcj_shell::commands::ids;
use jcj_shell::egui;
use jcj_shell::tree::{TreeEvent, TreeState};
use jcj_shell::{About, App, Cmd, CommandInfo, Commands, PanelSpec, Shell, ShellOptions, Side, TemplateSpec};
use serde_json::{Value, json};

use doc::OpenBook;
use outline::Key;

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
    /// The open book, if any.
    pub book: Option<OpenBook>,
    /// The outline's tree state.
    pub outline: TreeState<Key>,
    /// The open dialog's path field, while it shows.
    pub open_dialog: Option<String>,
    /// The import review, while it shows.
    pub review: Option<import_review::Review>,
    /// The last import's report.
    pub last_report: Option<wordsmith_book_import::Report>,
    /// The cleanup panel.
    pub cleanup: cleanup::Panel,
    /// The details panel.
    pub details: metadata::Panel,
    /// The design chooser.
    pub chooser: chooser::Chooser,
    /// A paragraph being fixed: its section, its number and the text so far.
    pub fixing: Option<(wordsmith_book_model::SectionId, usize, String)>,
    /// The last message for the user.
    pub message: Option<String>,
    revision: u64,
}

fn today() -> String {
    // The project's date: set once when the project is made, then kept (plan EXP-2).
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let days = i64::try_from(secs / 86_400).unwrap_or(0);
    // Civil date from days since 1970-01-01.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

fn str_param<'a>(params: &'a Value, key: &str) -> Result<&'a str, String> {
    params.get(key).and_then(Value::as_str).ok_or_else(|| format!("`{key}` is needed"))
}

impl BookApp {
    /// An app with no book open.
    pub fn new() -> BookApp {
        BookApp::default()
    }

    /// The open book, or an error saying none is.
    pub fn open_book(&mut self) -> Result<&mut OpenBook, String> {
        self.book.as_mut().ok_or_else(|| "No book is open.".to_owned())
    }

    /// Open the project at `path`.
    pub fn open_path(&mut self, path: &str) -> Result<(), String> {
        let b = OpenBook::open(&PathBuf::from(path))?;
        self.message = b.notice.clone();
        self.book = Some(b);
        self.outline = TreeState::default();
        self.revision += 1;
        Ok(())
    }

    /// Run one of Book's commands without a window (agents, the command line and tests use
    /// this; the window's [`App::run`] calls it too). Returns what the command gives back.
    pub fn command(&mut self, id: &str, params: &Value) -> Result<Value, String> {
        let out = commands::run(self, id, params)?;
        self.revision += 1;
        Ok(out)
    }

    fn draw_open_dialog(&mut self, ctx: &egui::Context, shell: &mut Shell) {
        let Some(mut path) = self.open_dialog.take() else { return };
        let mut keep = true;
        egui::Window::new("Open a Book").collapsible(false).resizable(false).show(ctx, |ui| {
            let label = ui.label("The path of a book project (.wsbk folder or .wsbkz file):");
            let r = ui.text_edit_singleline(&mut path).labelled_by(label.id);
            ui.horizontal(|ui| {
                let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if ui.button("Open").clicked() || enter {
                    shell.run(Cmd::with(ids::OPEN_PATH, json!({ "path": path.clone() })));
                    keep = false;
                }
                if ui.button("Cancel").clicked() {
                    keep = false;
                }
            });
        });
        if keep {
            self.open_dialog = Some(path);
        }
    }

    fn fix_editor(&mut self, ctx: &egui::Context) {
        let Some((section, n, mut text)) = self.fixing.take() else { return };
        let mut keep = true;
        let mut run = false;
        egui::Window::new("Fix Words").collapsible(false).show(ctx, |ui| {
            let label = ui.label("Change the words of this paragraph. A linked manuscript gets the fix as an offer the writer decides on.");
            ui.add(egui::TextEdit::multiline(&mut text).desired_width(520.0)).labelled_by(label.id);
            ui.horizontal(|ui| {
                run = ui.button("Make Fix").clicked();
                if ui.button("Cancel").clicked() {
                    keep = false;
                }
            });
        });
        if run {
            keep = false;
            match self.command(commands::FIX_EDIT, &json!({ "section": section.as_text(), "paragraph": n, "text": text.clone() })) {
                Ok(_) => self.message = Some("The fix was made.".into()),
                Err(m) => self.message = Some(m),
            }
        }
        if keep {
            self.fixing = Some((section, n, text));
        }
    }

    fn outline_ui(&mut self, ui: &mut egui::Ui) {
        let Some(b) = &self.book else {
            ui.label("No book open.");
            return;
        };
        let rows = outline::rows(&b.files.project, b.source.as_ref().ok().map(|s| &s.book));
        let events = jcj_shell::tree::tree(ui, "book-outline", &mut self.outline, &rows);
        for e in events {
            let result = match e {
                TreeEvent::Select(k) => {
                    self.outline.selected = Some(k);
                    Ok(Value::Null)
                }
                TreeEvent::Drop { item: Key::Element(id), target, zone } => self
                    .command(commands::MOVE_ELEMENT, &json!({ "id": id, "target": commands::key_json(&target), "zone": commands::zone_name(zone) })),
                TreeEvent::Drop { .. } => Err(outline::READ_ONLY.to_owned()),
                _ => Ok(Value::Null),
            };
            if let Err(m) = result {
                self.message = Some(m);
            }
        }
    }
}

impl Commands for BookApp {
    fn command(&self, id: &str) -> Option<CommandInfo> {
        self.all_commands().into_iter().find(|c| c.id == id)
    }

    fn all_commands(&self) -> Vec<CommandInfo> {
        commands::list()
    }
}

impl App for BookApp {
    fn document_name(&self) -> String {
        self.book.as_ref().map_or_else(|| "No book open".into(), OpenBook::name)
    }

    fn document_path(&self) -> Option<String> {
        self.book.as_ref().map(|b| b.root.display().to_string())
    }

    fn is_dirty(&self) -> bool {
        self.book.as_ref().is_some_and(OpenBook::is_dirty)
    }

    fn has_file(&self) -> bool {
        self.book.is_some()
    }

    fn revision(&self) -> u64 {
        self.revision + self.book.as_ref().map_or(0, |b| b.revision)
    }

    fn run(&mut self, cmd: &Cmd, shell: &mut Shell) -> Result<(), String> {
        match cmd.id.as_str() {
            ids::OPEN => {
                self.open_dialog = Some(String::new());
                Ok(())
            }
            ids::OPEN_PATH => {
                let path = str_param(&cmd.params, "path")?.to_owned();
                self.open_path(&path)?;
                jcj_shell::prefs::add_recent(&mut shell.recent, &path);
                Ok(())
            }
            id => self.command(id, &cmd.params).map(|_| ()),
        }
    }

    fn autosave(&mut self) -> Result<(), String> {
        match &mut self.book {
            Some(b) if b.is_dirty() => b.save(),
            _ => Ok(()),
        }
    }

    fn document_ui(&mut self, ui: &mut egui::Ui, shell: &mut Shell) {
        let ctx = ui.ctx().clone();
        brand::ensure_fonts(&ctx);
        self.draw_open_dialog(&ctx, shell);
        if let Some(b) = &mut self.book
            && ctx.input(|i| i.focused)
            && b.refresh_source()
        {
            self.message.clone_from(&b.notice);
        }
        if let Some(m) = self.message.clone() {
            let mut dismiss = false;
            ui.horizontal(|ui| {
                ui.label(m.as_str());
                dismiss = ui.small_button("Dismiss").clicked();
            });
            if dismiss {
                self.message = None;
            }
            ui.separator();
        }
        if let Some(review) = &mut self.review {
            let (go, cancel) = egui::ScrollArea::vertical().show(ui, |ui| review.ui(ui)).inner;
            if go {
                if let Err(m) = self.command(commands::IMPORT_CONFIRM, &json!({})) {
                    self.message = Some(m);
                }
            } else if cancel {
                self.review = None;
            }
            return;
        }
        let Some(b) = &self.book else {
            ui.centered_and_justified(|ui| ui.label("Open or create a book to begin."));
            return;
        };
        let selected = self.outline.selected.clone();
        let clicked = egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| views::document(ui, b, selected.as_ref())).inner;
        if let (Some((section, n)), Some(Key::Section(_))) = (clicked, &selected) {
            let old = b.source.as_ref().ok().and_then(|s| wordsmith_book_format::fixes::section(&s.book, section)).and_then(|s| {
                wordsmith_manuscript_model::Text { blocks: s.content.clone(), notes: vec![] }
                    .inline_runs()
                    .get(n)
                    .map(|r| wordsmith_manuscript_model::text::plain(r))
            });
            self.fixing = old.map(|t| (section, n, t));
        }
        self.fix_editor(&ctx);
    }

    fn panels(&self) -> Vec<PanelSpec> {
        vec![
            PanelSpec::new("outline", "Outline", "outline", Side::Left),
            PanelSpec::new("cleanup", "Cleanup", "draft", Side::Right),
            PanelSpec::new("details", "Details", "info", Side::Right),
            PanelSpec::new("designs", "Designs", "style", Side::Right),
        ]
    }

    fn panel_ui(&mut self, panel: &str, ui: &mut egui::Ui, _shell: &mut Shell) {
        brand::ensure_fonts(ui.ctx());
        match panel {
            "outline" => self.outline_ui(ui),
            "designs" => {
                let run = match &self.book {
                    Some(b) => self.chooser.ui(ui, b),
                    None => None,
                };
                if let Some((id, params)) = run
                    && let Err(m) = self.command(id, &params)
                {
                    self.message = Some(m);
                }
            }
            "details" => {
                let run = match &self.book {
                    Some(b) => self.details.ui(ui, b),
                    None => None,
                };
                if let Some((id, params)) = run
                    && let Err(m) = self.command(id, &params)
                {
                    self.message = Some(m);
                }
            }
            "cleanup" => {
                let run = match &self.book {
                    Some(b) => self.cleanup.ui(ui, b),
                    None => None,
                };
                if let Some((id, params)) = run
                    && let Err(m) = self.command(id, &params)
                {
                    self.message = Some(m);
                }
            }
            _ => {}
        }
    }

    fn templates(&self) -> Vec<TemplateSpec> {
        vec![
            TemplateSpec::new("link", "From a Manuscript project", "A book on a Wordsmith Manuscript project, read and never changed."),
            TemplateSpec::new("empty", "An empty book", "A book with its own source, to import a file into."),
            TemplateSpec::new("import", "Import a file", "A book from a DOCX, ODT, RTF, Markdown or text file, reviewed before import."),
        ]
    }

    fn mark_width(&self, height: f32) -> f32 {
        brand::mark_width(height)
    }

    fn paint_mark(&self, painter: &egui::Painter, rect: egui::Rect, ink: egui::Color32) {
        brand::paint_mark(painter, rect, ink);
    }

    fn has_document(&self) -> bool {
        self.book.is_some() || self.review.is_some() || self.open_dialog.is_some()
    }

    fn about(&self) -> About {
        About {
            summary: "Book design and typesetting: print-ready PDFs and accessible EPUBs from a finished manuscript.".into(),
            credit: "Built on open-source-core, based on WordCraft by the ArtCraft team.".into(),
            acknowledgments: "Bundled fonts are listed with their licenses in ATTRIBUTION.md.".into(),
            website: Some("https://github.com/jamescjfoundation/wordsmith-book".into()),
        }
    }
}

pub mod views {
    //! The central view: the selected section's text, read-only, or the book's summary.

    use jcj_shell::egui;
    use wordsmith_manuscript_model::Block;

    use crate::doc::OpenBook;
    use crate::outline::Key;

    fn find(
        sections: &[wordsmith_manuscript_format::book::Section],
        id: wordsmith_book_model::SectionId,
    ) -> Option<&wordsmith_manuscript_format::book::Section> {
        for s in sections {
            if s.id == id {
                return Some(s);
            }
            if let Some(found) = find(&s.children, id) {
                return Some(found);
            }
        }
        None
    }

    /// Draw the document area; gives the paragraph whose fix button was clicked.
    pub fn document(ui: &mut egui::Ui, b: &OpenBook, selected: Option<&Key>) -> Option<(wordsmith_book_model::SectionId, usize)> {
        let mut clicked = None;
        let book = b.source.as_ref().ok().map(|s| &s.book);
        match (selected, book) {
            (Some(Key::Section(id)), Some(book)) => {
                let all: Vec<_> = book.front.iter().chain(&book.body).chain(&book.back).cloned().collect();
                if let Some(s) = find(&all, *id) {
                    ui.heading(if s.title.is_empty() { "Untitled" } else { s.title.as_str() });
                    let mut n = 0usize;
                    for blk in &s.content {
                        match blk {
                            Block::Para(c) | Block::Heading { content: c, .. } => {
                                ui.horizontal_wrapped(|ui| {
                                    if ui.small_button("Fix").on_hover_text("Fix words in this paragraph").clicked() {
                                        clicked = Some((s.id, n));
                                    }
                                    ui.label(wordsmith_manuscript_model::text::plain(c));
                                });
                                n += 1;
                            }
                            Block::SceneBreak => {
                                ui.vertical_centered(|ui| ui.label("* * *"));
                            }
                            Block::Quote(ps) | Block::Fenced { paras: ps, .. } => {
                                for p in ps {
                                    ui.label(format!("    {}", wordsmith_manuscript_model::text::plain(p)));
                                    n += 1;
                                }
                            }
                            Block::Image { alt, .. } => {
                                ui.label(format!("[Picture: {alt}]"));
                            }
                            Block::Raw { .. } => {}
                        }
                    }
                }
            }
            (Some(Key::Element(id)), _) => {
                if let Some(e) = b.files.project.elements.iter().find(|e| &e.id == id) {
                    ui.heading(if e.title.is_empty() { e.kind.map_or("Untitled", |k| k.title()) } else { e.title.as_str() });
                    ui.label(if e.text.is_empty() { "Made from the book's details when the book is built." } else { e.text.as_str() });
                }
            }
            _ => {
                ui.heading(b.name());
                match book {
                    Some(book) => {
                        ui.label(format!("{} by {}", book.metadata.info.title, book.metadata.byline));
                        for (role, n) in wordsmith_manuscript_format::book::counts(book) {
                            ui.label(format!("{role}: {n}"));
                        }
                    }
                    None => {
                        ui.label("The book's source is not open.");
                    }
                }
            }
        }
        clicked
    }
}
