// SPDX-License-Identifier: MIT OR Apache-2.0
//! The Export page (EXP-1): which editions, the project bundle, and where; the editions are
//! built and checked on a worker thread and written only when nothing blocks export.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use jcj_shell::egui;
use wordsmith_book_check::export::{self, Built, Error};

use crate::doc::OpenBook;

/// The page's choices and its running export.
#[derive(Default)]
pub struct Page {
    /// Editions left out (all are chosen until one is turned off).
    pub off: BTreeSet<String>,
    /// Write the project bundle too.
    pub bundle: bool,
    /// The folder to write into (empty: an `Export` folder beside the project).
    pub folder: String,
    /// What the last export did, in words.
    pub last: Option<String>,
    job: Option<Receiver<Result<Vec<Built>, Error>>>,
}

/// The folder exports go to by default: `<project name> Export` beside the project.
pub fn default_folder(b: &OpenBook) -> PathBuf {
    let parent = b.root.parent().map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    parent.join(format!("{} Export", b.name()))
}

/// Export `editions` (and the bundle) of the open book into `folder` on this thread; gives the
/// paths written.
pub fn run(
    b: &OpenBook,
    editions: &[String],
    bundle: bool,
    folder: &Path,
    validator: Option<&wordsmith_book_check::validator::Settings>,
) -> Result<Vec<PathBuf>, String> {
    let src = b.source.as_ref().map_err(Clone::clone)?;
    let built = export::build_all(&b.files.project, &b.files.designs, src, editions, &|p| b.picture(p), validator).map_err(|e| e.to_string())?;
    write(b, built, bundle, folder)
}

fn write(b: &OpenBook, built: Vec<Built>, bundle: bool, folder: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files: Vec<(String, Vec<u8>)> = built.into_iter().map(|x| (x.name, x.bytes)).collect();
    if bundle {
        let bytes = wordsmith_book_format::pack::pack_folder(&b.root, &b.files).map_err(|e| e.to_string())?;
        files.push((format!("{}.wsbkz", export::file_name(&b.name())), bytes));
    }
    export::write(folder, &files).map_err(|e| e.to_string())
}

impl Page {
    /// Whether an export is running.
    pub fn running(&self) -> bool {
        self.job.is_some()
    }

    /// The editions chosen.
    pub fn chosen(&self, b: &OpenBook) -> Vec<String> {
        b.files.project.editions.iter().filter(|e| !self.off.contains(&e.id)).map(|e| e.id.clone()).collect()
    }

    fn start(&mut self, b: &OpenBook, validator: wordsmith_book_check::validator::Settings) {
        let Ok(src) = &b.source else { return };
        let (project, designs, src) = (b.files.project.clone(), b.files.designs.clone(), src.clone());
        let editions = self.chosen(b);
        let book = wordsmith_book_format::fixes::book_for_build(&project.fixes, &src);
        let mut paths: Vec<String> = crate::preview::pictures_of(&book);
        paths.extend(project.editions.iter().map(|e| e.cover.clone()).filter(|c| !c.is_empty()));
        let pictures: std::collections::BTreeMap<String, Vec<u8>> = paths.into_iter().filter_map(|p| b.picture(&p).map(|x| (p, x))).collect();
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let out = export::build_all(&project, &designs, &src, &editions, &|p| pictures.get(p).cloned(), Some(&validator));
            let _ = tx.send(out);
        });
        self.job = Some(rx);
        self.last = None;
    }

    /// Take a finished export and write its files (called every frame, so an export finishes
    /// while the page is closed).
    pub fn poll(&mut self, b: &OpenBook) {
        let Some(rx) = &self.job else { return };
        let Ok(out) = rx.try_recv() else { return };
        self.job = None;
        let folder = if self.folder.trim().is_empty() { default_folder(b) } else { PathBuf::from(self.folder.trim()) };
        self.last = Some(match out.map_err(|e| e.to_string()).and_then(|built| write(b, built, self.bundle, &folder)) {
            Ok(paths) => format!("Exported {} file{} to {}.", paths.len(), if paths.len() == 1 { "" } else { "s" }, folder.display()),
            Err(e) => capital(&e),
        });
    }

    /// Wait for the running export (tests).
    pub fn wait(&mut self, b: &OpenBook, limit: Duration) -> bool {
        let start = std::time::Instant::now();
        while self.job.is_some() && start.elapsed() < limit {
            self.poll(b);
            std::thread::sleep(Duration::from_millis(5));
        }
        self.job.is_none()
    }

    /// Draw the page.
    pub fn ui(&mut self, ui: &mut egui::Ui, b: &OpenBook, validator: &wordsmith_book_check::validator::Settings) {
        self.poll(b);
        ui.label("Each edition is built, checked word for word and preflighted first; nothing is written if anything blocks export.");
        ui.add_space(6.0);
        for e in &b.files.project.editions {
            let mut on = !self.off.contains(&e.id);
            let kind = if e.format.is_print() { "print PDF" } else { "EPUB" };
            if ui.checkbox(&mut on, format!("{} ({kind})", e.name)).changed() {
                if on {
                    self.off.remove(&e.id);
                } else {
                    self.off.insert(e.id.clone());
                }
            }
        }
        ui.checkbox(&mut self.bundle, "Project bundle (the book, its designs, pictures and source in one file)");
        ui.horizontal(|ui| {
            let label = ui.label("Folder");
            let hint = default_folder(b).display().to_string();
            ui.add(egui::TextEdit::singleline(&mut self.folder).hint_text(hint).desired_width(420.0)).labelled_by(label.id);
        });
        let nothing = self.chosen(b).is_empty() && !self.bundle;
        if ui.add_enabled(!self.running() && !nothing, egui::Button::new("Export Files")).clicked() {
            self.start(b, validator.clone());
        }
        if self.running() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Building and checking…");
            });
            ui.ctx().request_repaint_after(Duration::from_millis(50));
        }
        if let Some(m) = &self.last {
            ui.label(m.as_str());
        }
    }
}

fn capital(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}
