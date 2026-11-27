// SPDX-License-Identifier: MIT OR Apache-2.0
//! Packed Book projects (`.wsbkz`), recovery and backups (plan section 6.2), over the core's
//! folder layer (`jcj_project::folder`, core decision 0018).
//!
//! A packed project is a zip: `mimetype` first and stored, then every file in name order with
//! fixed times, so the same project packs to the same bytes. It holds the project's files, the
//! owned source's files under `source.wsm/`, and, for a linked source, a packed copy of it
//! (`linked.wsmz`, read-only) so the project travels whole. Reading never trusts the file: sizes
//! and counts are capped and entries are read one by one into memory, never extracted. What is
//! Book's here is what goes into a packed project ([`Packed`], [`gather`]); the zip, the limits,
//! recovery and backups are the core's.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use jcj_project::folder as core;
use wordsmith_book_model::Source;

use crate::folder::{FORMAT, Files, Folder, Saved, safe_name};
use crate::{Error, source};

/// The `mimetype` entry of a packed project.
pub const MIMETYPE: &str = "application/x-wordsmith-book-packed";
/// The packed copy of a linked source inside a packed project or a project folder.
pub const LINKED_COPY: &str = "linked.wsmz";

/// A packed project's contents.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Packed {
    /// The project's files.
    pub files: Files,
    /// The owned source's files, by path inside its folder.
    pub owned: BTreeMap<String, Vec<u8>>,
    /// A packed copy of a linked source.
    pub linked: Option<Vec<u8>>,
}

/// The folder an owned source's files are packed under: the owned source's name, or the name
/// a new owned source would take.
fn owned_dir(files: &Files) -> &str {
    match &files.project.source {
        Source::Owned(name) => name,
        Source::Manuscript(_) => wordsmith_book_model::OWNED_SOURCE,
    }
}

/// Every file of a packed project, by its path inside it.
fn entries(p: &Packed) -> Result<BTreeMap<String, Vec<u8>>, Error> {
    let mut entries = p.files.render()?;
    let owned_dir = owned_dir(&p.files);
    for (rel, bytes) in &p.owned {
        if !safe_name(rel) {
            return Err(Error::Write(format!("`{rel}` is not a usable file name in the source")));
        }
        entries.insert(format!("{owned_dir}/{rel}"), bytes.clone());
    }
    if let Some(bytes) = &p.linked {
        entries.insert(LINKED_COPY.to_owned(), bytes.clone());
    }
    Ok(entries)
}

/// Pack `p` into `.wsbkz` bytes; the same contents always give the same bytes.
pub fn pack(p: &Packed) -> Result<Vec<u8>, Error> {
    core::pack(&FORMAT, &entries(p)?).map_err(|e| Error::from_core(e, "the packed project"))
}

/// What is needed to pack the project in `root`: its files, its owned source's files, and a
/// packed copy of a linked source when it can be read.
pub fn gather(root: &Path, files: &Files) -> Result<Packed, Error> {
    let mut p = Packed { files: files.clone(), ..Packed::default() };
    let limit = FORMAT.limits.entry;
    match &files.project.source {
        Source::Owned(name) => {
            let dir = root.join(name);
            if dir.is_dir() {
                p.owned = wordsmith_manuscript_format::pack::folder_files(&dir).map_err(|e| Error::Source(e.to_string()))?;
            }
        }
        Source::Manuscript(_) => {
            let path = source::resolve(root, &files.project.source);
            if path.is_dir() {
                p.linked = Some(wordsmith_manuscript_format::pack::pack_folder(&path).map_err(|e| Error::Source(e.to_string()))?);
            } else if path.is_file() {
                p.linked = Some(jcj_project::files::read_bounded(&path, limit).map_err(|e| Error::Source(e.to_string()))?);
            } else if root.join(LINKED_COPY).is_file() {
                p.linked = Some(jcj_project::files::read_bounded(&root.join(LINKED_COPY), limit).map_err(|e| Error::Source(e.to_string()))?);
            }
        }
    }
    Ok(p)
}

/// Pack the project in `root` (Save as one file, backups, the project bundle).
pub fn pack_folder(root: &Path, files: &Files) -> Result<Vec<u8>, Error> {
    pack(&gather(root, files)?)
}

/// What a packed project's files hold for Book.
fn packed_of(map: &BTreeMap<String, Vec<u8>>) -> Result<Packed, Error> {
    let files = Files::from_map(map)?;
    let prefix = format!("{}/", owned_dir(&files));
    let owned = map.iter().filter_map(|(k, v)| k.strip_prefix(&prefix).map(|rel| (rel.to_owned(), v.clone()))).collect();
    let linked = map.get(LINKED_COPY).cloned();
    Ok(Packed { files, owned, linked })
}

/// Read a packed project, entry by entry, within the limits.
pub fn unpack(bytes: &[u8]) -> Result<Packed, Error> {
    let map = core::unpack(&FORMAT, bytes).map_err(|e| Error::from_core(e, "the packed project"))?;
    packed_of(&map)
}

/// Unpack a packed project into a new project folder at `dest` (which must be empty or absent)
/// and open it. Nothing is written until the whole file has been read and Book has read the
/// project from memory.
pub fn unpack_into(bytes: &[u8], dest: &Path) -> Result<(Folder, Files), Error> {
    let mut refused = None;
    let written = core::unpack_into(&FORMAT, bytes, dest, |map| match packed_of(map) {
        Ok(_) => Ok(()),
        Err(e) => {
            let why = e.to_string();
            refused = Some(e);
            Err(jcj_project::Error::Damaged(why))
        }
    });
    if let Some(e) = refused {
        return Err(e);
    }
    written.map_err(|e| Error::from_core(e, "the packed project"))?;
    Folder::open(dest)
}

/// Write `bytes` to `path` atomically (Save as one file, backups).
pub fn write_file(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    jcj_project::files::write_atomic(path, bytes).map_err(|e| Error::Io { file: path.display().to_string(), why: e.to_string() })
}

/// The recovery file of the project folder at `root`: `.recovery/unsaved.wsbkz`.
pub fn recovery_path(root: &Path) -> PathBuf {
    core::recovery_path(&FORMAT, root)
}

/// Keep unsaved work beside the project (the project's own files only; the source is not copied).
pub fn write_recovery(root: &Path, files: &Files) -> Result<(), Error> {
    core::write_recovery(&FORMAT, root, &files.render()?).map_err(|e| Error::from_core(e, "the recovery copy"))
}

/// The unsaved work kept beside the project, if any and if it reads.
pub fn recovered(root: &Path) -> Option<Files> {
    Files::from_map(&core::recovered(&FORMAT, root)?).ok()
}

/// Remove the recovery file, and the recovery folder when nothing else is in it.
pub fn discard_recovery(root: &Path) -> Result<(), Error> {
    core::discard_recovery(&FORMAT, root).map_err(|e| Error::from_core(e, "the recovery copy"))
}

/// Save with recovery around it: the work is kept beside the project first, the changed files
/// are written, then the recovery file is removed. A crash at any point reopens to the last good
/// save with the work recoverable, or to the new save.
pub fn save_safely(folder: &mut Folder, files: &Files) -> Result<Saved, Error> {
    folder.save_safely(files)
}

/// Backups of `name` in `dir`, oldest first: only the files named as its backups.
pub fn backups(dir: &Path, name: &str) -> Vec<PathBuf> {
    core::backups(&FORMAT, dir, name)
}

/// Write a packed backup of the project in `root` to `dir` as `<name> <YYYY-MM-DD HH.MM.SS>
/// UTC.wsbkz` (a number added for a second backup in the same second, never over a file),
/// keeping the newest `keep`.
pub fn backup(root: &Path, files: &Files, dir: &Path, name: &str, keep: usize, at: SystemTime) -> Result<PathBuf, Error> {
    let bytes = pack_folder(root, files)?;
    core::backup_packed(&FORMAT, &bytes, dir, name, keep, at).map_err(|e| Error::from_core(e, "the backup"))
}
