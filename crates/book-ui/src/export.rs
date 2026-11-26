// SPDX-License-Identifier: MIT OR Apache-2.0
//! The Export page (EXP-1): which editions, the project bundle, and where; the editions are
//! built and checked on a worker thread and written only when nothing blocks export.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use jcj_shell::egui;
use jcj_shell::i18n::{tr, tr_with, trn};
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
    /// A proof asked for on the page, by edition id (the app writes it).
    pub proof_request: Option<String>,
    /// The distributor record asked for on the page (the app writes it).
    pub onix_request: bool,
    job: Option<Receiver<Result<Vec<Built>, Error>>>,
}

/// The folder exports go to by default: `<project name> Export` beside the project.
pub fn default_folder(b: &OpenBook) -> PathBuf {
    let parent = b.root.parent().map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    parent.join(format!("{} Export", b.name()))
}

/// Write the book's ONIX for Books 3.0 record into `folder` (META-6), one product for each
/// edition (print editions laid out for their page counts): `<title> ONIX.xml`; gives its path.
pub fn onix(b: &OpenBook, folder: &Path) -> Result<PathBuf, String> {
    let src = b.source.as_ref().map_err(Clone::clone)?;
    let p = &b.files.project;
    let book = wordsmith_book_format::fixes::book_for_build(&p.fixes, src);
    let mut pages = Vec::with_capacity(p.editions.len());
    for e in &p.editions {
        pages.push(if e.format.is_print() {
            Some(wordsmith_book_check::build::print(p, &b.files.designs, &book, e, &|x| b.picture(x))?.typeset.page_count())
        } else {
            None
        });
    }
    let records: Vec<wordsmith_book_check::onix::Record<'_>> =
        p.editions.iter().zip(pages).map(|(edition, pages)| wordsmith_book_check::onix::Record { edition, pages }).collect();
    let meta = wordsmith_book_model::matter::meta(&book.metadata.info, &book.metadata.byline, &p.metadata, None);
    let sender = if meta.imprint.trim().is_empty() { meta.byline.clone() } else { meta.imprint.clone() };
    let xml = wordsmith_book_check::onix::message(p, &book, &records, &sender, if p.date.is_empty() { "2000-01-01" } else { &p.date });
    std::fs::create_dir_all(folder).map_err(|e| format!("{}: {e}", folder.display()))?;
    let title = p.metadata.overrides.get("title").cloned().unwrap_or_else(|| book.metadata.info.title.clone());
    let path = folder.join(format!("{}.xml", wordsmith_book_check::export::file_name(&format!("{title} ONIX"))));
    wordsmith_book_check::export::write_file(&path, xml.as_bytes())?;
    Ok(path)
}

/// Write a proof of print edition `edition` into `folder` (PRT-13): `<title> - <edition> proof.pdf`;
/// gives its path.
pub fn proof(b: &OpenBook, edition: &str, folder: &Path) -> Result<PathBuf, String> {
    let src = b.source.as_ref().map_err(Clone::clone)?;
    let e = b.files.project.edition(edition).ok_or_else(|| format!("there is no edition `{edition}`"))?;
    if !e.format.is_print() {
        return Err(format!("the {} is not printed, so it has no proof", e.name.to_lowercase()));
    }
    let book = wordsmith_book_format::fixes::book_for_build(&b.files.project.fixes, src);
    let pr = wordsmith_book_check::build::proof(&b.files.project, &b.files.designs, &book, e, &|p| b.picture(p))?;
    let pdf = pr.typeset.pdf().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(folder).map_err(|e| format!("{}: {e}", folder.display()))?;
    let title = b.files.project.metadata.overrides.get("title").cloned().unwrap_or_else(|| book.metadata.info.title.clone());
    let path = folder.join(format!("{}.pdf", wordsmith_book_check::export::file_name(&format!("{title} - {} proof", e.name))));
    wordsmith_book_check::export::write_file(&path, &pdf)?;
    Ok(path)
}

/// Why an export wrote nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stopped {
    /// A check of the content blocks export: the edition's preflight report.
    Blocked(Box<wordsmith_book_check::preflight::Report>),
    /// The export could not be done: the source is not open, an edition could not be built, a
    /// file could not be written.
    Failed(String),
}

impl std::fmt::Display for Stopped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stopped::Blocked(report) => write!(f, "{}", Error::Blocked(report.clone())),
            Stopped::Failed(why) => f.write_str(why),
        }
    }
}

/// Export `editions` (and the bundle) of the open book into `folder` on this thread; gives the
/// paths written, or why nothing was (the command line tells a blocked export from a failure).
pub fn run_checked(
    b: &OpenBook,
    editions: &[String],
    bundle: bool,
    folder: &Path,
    validator: Option<&wordsmith_book_check::validator::Settings>,
) -> Result<Vec<PathBuf>, Stopped> {
    let src = b.source.as_ref().map_err(|e| Stopped::Failed(e.clone()))?;
    let built = export::build_all(&b.files.project, &b.files.designs, src, editions, &|p| b.picture(p), validator).map_err(|e| match e {
        Error::Blocked(report) => Stopped::Blocked(report),
        e => Stopped::Failed(e.to_string()),
    })?;
    write(b, built, bundle, folder).map_err(Stopped::Failed)
}

/// [`run_checked`], with the reason as text.
pub fn run(
    b: &OpenBook,
    editions: &[String],
    bundle: bool,
    folder: &Path,
    validator: Option<&wordsmith_book_check::validator::Settings>,
) -> Result<Vec<PathBuf>, String> {
    run_checked(b, editions, bundle, folder, validator).map_err(|e| e.to_string())
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
            Ok(paths) => trn(
                "Exported {n} file to {folder}.",
                "Exported {n} files to {folder}.",
                paths.len() as u64,
                &[("folder", &folder.display().to_string())],
            ),
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
        ui.label(tr("Each edition is built, checked word for word and preflighted first; nothing is written if anything blocks export."));
        ui.add_space(6.0);
        for e in &b.files.project.editions {
            let mut on = !self.off.contains(&e.id);
            let kind = if e.format.is_print() { tr("print PDF") } else { tr("EPUB") };
            if ui.checkbox(&mut on, format!("{} ({kind})", e.name)).changed() {
                if on {
                    self.off.remove(&e.id);
                } else {
                    self.off.insert(e.id.clone());
                }
            }
        }
        ui.checkbox(&mut self.bundle, tr("Project bundle (the book, its designs, pictures and source in one file)"));
        ui.horizontal(|ui| {
            let label = ui.label(tr("Folder"));
            let hint = default_folder(b).display().to_string();
            ui.add(egui::TextEdit::singleline(&mut self.folder).hint_text(hint).desired_width(420.0)).labelled_by(label.id);
        });
        let nothing = self.chosen(b).is_empty() && !self.bundle;
        if ui.add_enabled(!self.running() && !nothing, egui::Button::new(tr("Export Files"))).clicked() {
            self.start(b, validator.clone());
        }
        if self.running() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(tr("Building and checking…"));
            });
            ui.ctx().request_repaint_after(Duration::from_millis(50));
        }
        if let Some(m) = &self.last {
            ui.label(m.as_str());
        }
        // The distributor record (META-6).
        ui.separator();
        ui.label(tr("Distributors and stores read a book's details from an ONIX for Books record: one product for each edition."));
        if ui.add_enabled(!self.running(), egui::Button::new(tr("Write the ONIX Record"))).clicked() {
            self.onix_request = true;
        }
        // Proofs (PRT-13): never part of the export above; one is written only when asked for.
        let print: Vec<_> = b.files.project.editions.iter().filter(|e| e.format.is_print()).collect();
        if !print.is_empty() {
            ui.separator();
            ui.label(tr(
                "A proof is the print PDF with PROOF across every page and its lines numbered, to read and mark; it is never sent to a printer.",
            ));
            for e in print {
                if ui.add_enabled(!self.running(), egui::Button::new(tr_with("Write a Proof of the {edition}", &[("edition", &e.name)]))).clicked() {
                    self.proof_request = Some(e.id.clone());
                }
            }
        }
    }
}

fn capital(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}
