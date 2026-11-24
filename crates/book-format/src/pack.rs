// SPDX-License-Identifier: MIT OR Apache-2.0
//! Packed Book projects (`.wsbkz`), recovery and backups (plan section 6.2, core decision 0009).
//!
//! A packed project is a zip: `mimetype` first and stored, then every file in name order with
//! fixed times, so the same project packs to the same bytes. It holds the project's files, the
//! owned source's files under `source.wsm/`, and, for a linked source, a packed copy of it
//! (`linked.wsmz`, read-only) so the project travels whole. Reading never trusts the file: sizes
//! and counts are capped and entries are read one by one into memory, never extracted.

use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use wordsmith_book_model::Source;

use crate::folder::{Files, Folder, Saved, safe_name};
use crate::{Error, source};

/// The `mimetype` entry of a packed project.
pub const MIMETYPE: &str = "application/x-wordsmith-book-packed";
/// The packed copy of a linked source inside a packed project or a project folder.
pub const LINKED_COPY: &str = "linked.wsmz";
/// The recovery folder inside a project folder.
pub const RECOVERY_DIR: &str = ".recovery";
/// The recovery file's name.
pub const RECOVERY_FILE: &str = "unsaved.wsbkz";
/// The largest packed project read.
pub const MAX_PACKED: u64 = 2 << 30;
/// The largest entry.
pub const MAX_ENTRY: u64 = 512 << 20;
/// The most entries.
pub const MAX_ENTRIES: usize = 50_000;
/// All entries together, unpacked.
pub const MAX_UNPACKED: u64 = 4 << 30;

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

fn zip_err(e: impl std::fmt::Display) -> Error {
    Error::Damaged { file: "the packed project".into(), why: e.to_string() }
}

/// Pack `p` into `.wsbkz` bytes; the same contents always give the same bytes.
pub fn pack(p: &Packed) -> Result<Vec<u8>, Error> {
    let mut entries: BTreeMap<String, Vec<u8>> = p.files.render()?;
    let owned_dir = match &p.files.project.source {
        Source::Owned(name) => name.clone(),
        Source::Manuscript(_) => wordsmith_book_model::OWNED_SOURCE.to_owned(),
    };
    for (rel, bytes) in &p.owned {
        if !safe_name(rel) {
            return Err(Error::Write(format!("`{rel}` is not a usable file name in the source")));
        }
        entries.insert(format!("{owned_dir}/{rel}"), bytes.clone());
    }
    if let Some(bytes) = &p.linked {
        entries.insert(LINKED_COPY.to_owned(), bytes.clone());
    }
    let mut w = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let fixed = zip::DateTime::default();
    let stored =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored).last_modified_time(fixed).unix_permissions(0o644);
    let deflated = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(fixed)
        .unix_permissions(0o644);
    w.start_file("mimetype", stored).map_err(|e| Error::Write(e.to_string()))?;
    w.write_all(MIMETYPE.as_bytes()).map_err(|e| Error::Write(e.to_string()))?;
    for (name, bytes) in &entries {
        let opts = if name.ends_with(".wsmz") || name.ends_with(".png") || name.ends_with(".jpg") { stored } else { deflated };
        w.start_file(name.as_str(), opts).map_err(|e| Error::Write(e.to_string()))?;
        w.write_all(bytes).map_err(|e| Error::Write(e.to_string()))?;
    }
    Ok(w.finish().map_err(|e| Error::Write(e.to_string()))?.into_inner())
}

/// What is needed to pack the project in `root`: its files, its owned source's files, and a
/// packed copy of a linked source when it can be read.
pub fn gather(root: &Path, files: &Files) -> Result<Packed, Error> {
    let mut p = Packed { files: files.clone(), ..Packed::default() };
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
                p.linked = Some(jcj_project::files::read_bounded(&path, MAX_ENTRY).map_err(|e| Error::Source(e.to_string()))?);
            } else if root.join(LINKED_COPY).is_file() {
                p.linked = Some(jcj_project::files::read_bounded(&root.join(LINKED_COPY), MAX_ENTRY).map_err(|e| Error::Source(e.to_string()))?);
            }
        }
    }
    Ok(p)
}

/// Pack the project in `root` (Save as one file, backups, the project bundle).
pub fn pack_folder(root: &Path, files: &Files) -> Result<Vec<u8>, Error> {
    pack(&gather(root, files)?)
}

/// Read a packed project, entry by entry, within the limits.
pub fn unpack(bytes: &[u8]) -> Result<Packed, Error> {
    if bytes.len() as u64 > MAX_PACKED {
        return Err(Error::TooLarge { what: "the packed project".into(), limit: MAX_PACKED });
    }
    let mut z = zip::ZipArchive::new(Cursor::new(bytes)).map_err(zip_err)?;
    if z.len() > MAX_ENTRIES {
        return Err(Error::TooLarge { what: "the packed project's entries".into(), limit: MAX_ENTRIES as u64 });
    }
    let mut total = 0u64;
    let mut map: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for i in 0..z.len() {
        let mut f = z.by_index(i).map_err(zip_err)?;
        let name = f.name().to_owned();
        if f.is_dir() {
            continue;
        }
        if i == 0 && name != "mimetype" {
            return Err(zip_err("the first entry is not `mimetype`"));
        }
        if name != "mimetype" && !safe_name(&name) {
            return Err(zip_err(format!("the entry name `{}` is not allowed", name.chars().take(80).collect::<String>())));
        }
        let mut buf = Vec::new();
        (&mut f).take(MAX_ENTRY + 1).read_to_end(&mut buf).map_err(zip_err)?;
        if buf.len() as u64 > MAX_ENTRY {
            return Err(Error::TooLarge { what: name, limit: MAX_ENTRY });
        }
        total += buf.len() as u64;
        if total > MAX_UNPACKED {
            return Err(Error::TooLarge { what: "the packed project, unpacked".into(), limit: MAX_UNPACKED });
        }
        if map.insert(name.clone(), buf).is_some() {
            return Err(zip_err(format!("`{name}` is in the pack twice")));
        }
    }
    if map.get("mimetype").map(Vec::as_slice) != Some(MIMETYPE.as_bytes()) {
        return Err(zip_err("this is not a packed Wordsmith Book project"));
    }
    let files = Files::from_map(&map)?;
    let owned_dir = match &files.project.source {
        Source::Owned(name) => format!("{name}/"),
        Source::Manuscript(_) => format!("{}/", wordsmith_book_model::OWNED_SOURCE),
    };
    let owned = map.iter().filter_map(|(k, v)| k.strip_prefix(&owned_dir).map(|rel| (rel.to_owned(), v.clone()))).collect();
    let linked = map.get(LINKED_COPY).cloned();
    Ok(Packed { files, owned, linked })
}

/// Unpack a packed project into a new project folder at `dest` (which must be empty or absent)
/// and open it.
pub fn unpack_into(bytes: &[u8], dest: &Path) -> Result<(Folder, Files), Error> {
    let p = unpack(bytes)?;
    if dest.exists() && std::fs::read_dir(dest).map(|mut d| d.next().is_some()).unwrap_or(true) {
        return Err(Error::Io { file: dest.display().to_string(), why: "the folder is not empty".into() });
    }
    let folder = Folder::create(dest, &p.files)?;
    if let Source::Owned(name) = &p.files.project.source {
        let mut sink = crate::folder::Disk(dest);
        for (rel, bytes) in &p.owned {
            crate::folder::Sink::write(&mut sink, &format!("{name}/{rel}"), bytes)?;
        }
    }
    if let Some(bytes) = &p.linked {
        jcj_project::files::write_atomic(&dest.join(LINKED_COPY), bytes).map_err(|e| Error::Io { file: LINKED_COPY.into(), why: e.to_string() })?;
    }
    Ok((folder, p.files))
}

/// Write `bytes` to `path` atomically (Save as one file, backups).
pub fn write_file(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    jcj_project::files::write_atomic(path, bytes).map_err(|e| Error::Io { file: path.display().to_string(), why: e.to_string() })
}

/// The recovery file of the project folder at `root`.
pub fn recovery_path(root: &Path) -> PathBuf {
    root.join(RECOVERY_DIR).join(RECOVERY_FILE)
}

/// Keep unsaved work beside the project (the project's own files only; the source is not copied).
pub fn write_recovery(root: &Path, files: &Files) -> Result<(), Error> {
    let bytes = pack(&Packed { files: files.clone(), ..Packed::default() })?;
    let path = recovery_path(root);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| Error::Io { file: RECOVERY_DIR.into(), why: e.to_string() })?;
    }
    write_file(&path, &bytes)
}

/// The unsaved work kept beside the project, if any and if it reads.
pub fn recovered(root: &Path) -> Option<Files> {
    let bytes = jcj_project::files::read_bounded(&recovery_path(root), MAX_PACKED).ok()?;
    unpack(&bytes).ok().map(|p| p.files)
}

/// Remove the recovery file.
pub fn discard_recovery(root: &Path) -> Result<(), Error> {
    match std::fs::remove_file(recovery_path(root)) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Error::Io { file: RECOVERY_FILE.into(), why: e.to_string() }),
    }
}

/// Save with recovery around it: the work is kept beside the project first, the changed files
/// are written, then the recovery file is removed. A crash at any point reopens to the last good
/// save with the work recoverable, or to the new save.
pub fn save_safely(folder: &mut Folder, files: &Files) -> Result<Saved, Error> {
    let root = folder.root().to_path_buf();
    write_recovery(&root, files)?;
    let saved = folder.save(files)?;
    discard_recovery(&root)?;
    Ok(saved)
}

/// Backups of `name` in `dir`, oldest first.
pub fn backups(dir: &Path, name: &str) -> Vec<PathBuf> {
    let prefix = format!("{name} ");
    let suffix = format!(".{}", crate::PACKED_EXTENSION);
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with(&prefix) && n.ends_with(&suffix)))
        .collect();
    out.sort();
    out
}

/// Write a packed backup of the project in `root` to `dir` as `<name> <YYYY-MM-DD HH.MM.SS>
/// UTC.wsbkz` (a number added for a second backup in the same second), keeping the newest `keep`.
pub fn backup(root: &Path, files: &Files, dir: &Path, name: &str, keep: usize, at: SystemTime) -> Result<PathBuf, Error> {
    let bytes = pack_folder(root, files)?;
    std::fs::create_dir_all(dir).map_err(|e| Error::Io { file: dir.display().to_string(), why: e.to_string() })?;
    let stamp = jcj_project::files::stamp(at);
    let mut path = dir.join(format!("{name} {stamp}.{}", crate::PACKED_EXTENSION));
    let mut n = 2;
    while path.exists() && n < 1000 {
        path = dir.join(format!("{name} {stamp} {n}.{}", crate::PACKED_EXTENSION));
        n += 1;
    }
    write_file(&path, &bytes)?;
    let all = backups(dir, name);
    if all.len() > keep {
        for old in &all[..all.len() - keep] {
            let _ = std::fs::remove_file(old);
        }
    }
    Ok(path)
}
