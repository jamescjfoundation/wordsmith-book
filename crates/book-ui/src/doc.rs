// SPDX-License-Identifier: MIT OR Apache-2.0
//! An open Book project: its files, its source, undo and redo, saving with recovery, backups.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use wordsmith_book_format::source::{self, Opened};
use wordsmith_book_format::{Files, Folder, pack};
use wordsmith_book_model::{BookProject, Source};

/// How many undo steps are kept.
pub const MAX_UNDO: usize = 200;
/// How many backups are kept.
pub const KEEP_BACKUPS: usize = 10;

/// An open project.
pub struct OpenBook {
    /// The project folder.
    pub root: PathBuf,
    folder: Folder,
    /// The project's files now.
    pub files: Files,
    /// The source, or why it could not be opened.
    pub source: Result<Opened, String>,
    undo: Vec<Files>,
    redo: Vec<Files>,
    /// Changes so far (auto-save's clock).
    pub revision: u64,
    saved_revision: u64,
    /// A message for the user about the project (recovered work, a source that changed).
    pub notice: Option<String>,
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// The project folder a path names: a `.wsbk` folder, its `book.toml`, or a `.wsbkz` (unpacked
/// into a folder of the same name beside it the first time).
pub fn project_folder(path: &Path) -> Result<PathBuf, String> {
    if path.file_name().is_some_and(|n| n == "book.toml") {
        return path.parent().map(Path::to_path_buf).ok_or_else(|| "no project folder".to_owned());
    }
    if path.extension().is_some_and(|e| e == wordsmith_book_format::PACKED_EXTENSION) {
        let dest = path.with_extension(wordsmith_book_format::EXTENSION);
        if !dest.join("book.toml").exists() {
            let data = std::fs::read(path).map_err(err)?;
            pack::unpack_into(&data, &dest).map_err(err)?;
        }
        return Ok(dest);
    }
    Ok(path.to_path_buf())
}

impl OpenBook {
    /// Open the project at `path` (see [`project_folder`]). Unsaved work recovered from a crash
    /// is loaded and left unsaved, with a notice.
    pub fn open(path: &Path) -> Result<OpenBook, String> {
        let root = project_folder(path)?;
        let (folder, files) = Folder::open(&root).map_err(err)?;
        let source = source::open(&root, &files.project.source).map_err(err);
        let mut b = OpenBook { root, folder, files, source, undo: Vec::new(), redo: Vec::new(), revision: 0, saved_revision: 0, notice: None };
        if let Some(recovered) = pack::recovered(&b.root)
            && recovered != b.files
        {
            b.files = recovered;
            b.revision = 1;
            b.notice = Some("Unsaved work from last time was recovered. Save to keep it.".into());
        }
        if let Err(e) = &b.source {
            b.notice = Some(format!("The book's source could not be opened: {e}"));
        }
        Ok(b)
    }

    /// Create a project at `path` on `source` (a linked Manuscript project or an owned source
    /// written by an import) and open it.
    pub fn create(path: &Path, source: Source, date: &str) -> Result<OpenBook, String> {
        let files = Files::new(BookProject::new(source, date));
        Folder::create(path, &files).map_err(err)?;
        OpenBook::open(path)
    }

    /// The project's name (the folder's name without its extension).
    pub fn name(&self) -> String {
        self.root.file_stem().map_or_else(|| "Untitled".into(), |n| n.to_string_lossy().into_owned())
    }

    /// Whether there are unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.revision != self.saved_revision
    }

    /// Change the project's files as one undoable step. On an error nothing changes.
    pub fn edit(&mut self, f: impl FnOnce(&mut Files) -> Result<(), String>) -> Result<(), String> {
        let mut next = self.files.clone();
        f(&mut next)?;
        let problems = next.project.problems();
        if let Some(p) = problems.first() {
            return Err(p.clone());
        }
        if next == self.files {
            return Ok(());
        }
        self.undo.push(std::mem::replace(&mut self.files, next));
        if self.undo.len() > MAX_UNDO {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.revision += 1;
        Ok(())
    }

    /// Whether there is something to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Whether there is something to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Undo the last change.
    pub fn undo(&mut self) -> bool {
        match self.undo.pop() {
            Some(prev) => {
                self.redo.push(std::mem::replace(&mut self.files, prev));
                self.revision += 1;
                true
            }
            None => false,
        }
    }

    /// Redo the last undone change.
    pub fn redo(&mut self) -> bool {
        match self.redo.pop() {
            Some(next) => {
                self.undo.push(std::mem::replace(&mut self.files, next));
                self.revision += 1;
                true
            }
            None => false,
        }
    }

    /// Save the changed files, with recovery around the save.
    pub fn save(&mut self) -> Result<(), String> {
        pack::save_safely(&mut self.folder, &self.files).map_err(err)?;
        self.saved_revision = self.revision;
        self.notice = None;
        Ok(())
    }

    /// Keep the unsaved work beside the project (called while there are changes).
    pub fn write_recovery(&self) -> Result<(), String> {
        pack::write_recovery(&self.root, &self.files).map_err(err)
    }

    /// Save the whole project as one packed file at `path`.
    pub fn save_as_one_file(&self, path: &Path) -> Result<(), String> {
        let bytes = pack::pack_folder(&self.root, &self.files).map_err(err)?;
        pack::write_file(path, &bytes).map_err(err)
    }

    /// The folder backups go in: beside the project, `<name> Backups`.
    pub fn backup_dir(&self) -> PathBuf {
        let parent = self.root.parent().map_or_else(|| PathBuf::from("."), Path::to_path_buf);
        parent.join(format!("{} Backups", self.name()))
    }

    /// Write a packed backup, keeping the newest [`KEEP_BACKUPS`].
    pub fn backup(&self, at: SystemTime) -> Result<PathBuf, String> {
        pack::backup(&self.root, &self.files, &self.backup_dir(), &self.name(), KEEP_BACKUPS, at).map_err(err)
    }

    /// Read the source again if it changed on disk (FIL-2); true when it did.
    pub fn refresh_source(&mut self) -> bool {
        let changed = match &self.source {
            Ok(s) => s.changed_on_disk().unwrap_or(true),
            Err(_) => true,
        };
        if !changed {
            return false;
        }
        let old = self.source.as_ref().ok().map(|s| source::checksums(&s.book));
        self.source = source::open(&self.root, &self.files.project.source).map_err(err);
        match (&self.source, old) {
            (Ok(s), Some(old)) => {
                let n = source::changed_sections(&old, &source::checksums(&s.book)).len();
                if n > 0 {
                    self.notice = Some(format!("The manuscript changed: {n} section{} read again.", if n == 1 { "" } else { "s" }));
                }
            }
            (Err(e), _) => self.notice = Some(format!("The book's source could not be opened: {e}")),
            _ => {}
        }
        true
    }
}
