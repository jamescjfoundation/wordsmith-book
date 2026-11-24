// SPDX-License-Identifier: MIT OR Apache-2.0
//! The book's source (plan section 6.3): a linked Manuscript project, folder or packed, read
//! through Manuscript's book view and never written; or an owned source, a Manuscript-format
//! folder inside the Book project, written with Manuscript's public folder writer. Either way
//! Book has one input model: the book view.
//!
//! Each build records a checksum of every section's words, so Book can say what changed in the
//! text since the last export; a linked project changed on disk is noticed and read again
//! (FIL-2).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use wordsmith_book_model::{SectionId, Source};
use wordsmith_manuscript_format::book::{self, Book, Section};
use wordsmith_manuscript_model::{Project, Role, Text};

use crate::{Error, reads_book_view};

/// The kind of source that was opened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A linked Manuscript project folder.
    LinkedFolder,
    /// A linked packed Manuscript project (`.wsmz`).
    LinkedPacked,
    /// The owned source inside the Book project.
    Owned,
}

#[derive(Clone)]
enum Watch {
    Folder(wordsmith_manuscript_format::folder::Folder),
    Packed([u8; 32]),
}

/// An opened source: the book view, the project it came from (for the writer's decisions on
/// offers), and what is needed to notice outside changes. A clone is a snapshot (for work on
/// another thread).
#[derive(Clone)]
pub struct Opened {
    /// The book view.
    pub book: Book,
    /// The Manuscript project, as read.
    pub project: Project,
    /// What kind of source.
    pub kind: Kind,
    /// Where it is on disk.
    pub path: PathBuf,
    watch: Watch,
}

/// Where `source` is, for the project folder at `project_root`.
pub fn resolve(project_root: &Path, source: &Source) -> PathBuf {
    match source {
        Source::Manuscript(p) => project_root.join(p),
        Source::Owned(p) => project_root.join(p),
    }
}

/// Refuse a book view this build does not read.
pub fn check_version(b: &Book) -> Result<(), Error> {
    if reads_book_view(b.version) {
        Ok(())
    } else {
        Err(Error::Source(format!(
            "the Manuscript project gives book view version {}; this version of Wordsmith Book reads version {}. Update Wordsmith Book to open it.",
            b.version,
            crate::BOOK_VIEW_VERSIONS.iter().map(u32::to_string).collect::<Vec<_>>().join(", ")
        )))
    }
}

fn source_err(e: wordsmith_manuscript_format::Error) -> Error {
    Error::Source(e.to_string())
}

fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

/// Open the source of the project at `project_root`. Never writes anything.
pub fn open(project_root: &Path, source: &Source) -> Result<Opened, Error> {
    crate::check_source(source)?;
    let path = resolve(project_root, source);
    let owned = matches!(source, Source::Owned(_));
    if path.is_dir() {
        let (folder, project) = wordsmith_manuscript_format::folder::Folder::open(&path).map_err(source_err)?;
        let b = book::from_project(&project);
        check_version(&b)?;
        let kind = if owned { Kind::Owned } else { Kind::LinkedFolder };
        return Ok(Opened { book: b, project, kind, path, watch: Watch::Folder(folder) });
    }
    if owned {
        return Err(Error::Source(format!("the owned source `{}` is missing from the project", path.display())));
    }
    // A linked source that is not where the project says (a project opened from a bundle on
    // another machine) is read from the packed copy the project carries, if it has one.
    let copy = project_root.join(crate::pack::LINKED_COPY);
    let path = if !path.exists() && copy.is_file() { copy } else { path };
    let bytes = jcj_project::files::read_bounded(&path, wordsmith_manuscript_format::pack::MAX_PACKED)
        .map_err(|e| Error::Source(format!("{}: {e}", path.display())))?;
    let project = wordsmith_manuscript_format::pack::read_packed(&bytes).map_err(source_err)?;
    let b = book::from_project(&project);
    check_version(&b)?;
    Ok(Opened { book: b, project, kind: Kind::LinkedPacked, path, watch: Watch::Packed(digest(&bytes)) })
}

/// Largest picture read from a source, in bytes.
pub const MAX_PICTURE: u64 = 64 << 20;

/// Whether `src` is a picture path a source may hold: relative, under `media/`, no `..`.
pub fn picture_path_ok(src: &str) -> bool {
    src.starts_with("media/") && !src.contains("..") && !src.contains('\\') && !src.contains(':') && src.len() < 512
}

impl Opened {
    /// A picture of the source by its path (`media/<id>.png`), from its folder or its packed
    /// file; `None` if the path is not a picture path, or the file is missing or too large.
    pub fn picture(&self, src: &str) -> Option<Vec<u8>> {
        if !picture_path_ok(src) {
            return None;
        }
        if self.path.is_dir() {
            return jcj_project::files::read_bounded(&self.path.join(src), MAX_PICTURE).ok();
        }
        let bytes = jcj_project::files::read_bounded(&self.path, wordsmith_manuscript_format::pack::MAX_PACKED).ok()?;
        let mut files = wordsmith_manuscript_format::pack::unpack(&bytes).ok()?;
        files.remove(src).filter(|b| b.len() as u64 <= MAX_PICTURE)
    }

    /// Whether the source changed on disk since it was opened (FIL-2).
    pub fn changed_on_disk(&self) -> Result<bool, Error> {
        match &self.watch {
            Watch::Folder(f) => Ok(!f.outside_changes().map_err(source_err)?.is_empty()),
            Watch::Packed(h) => {
                let bytes = jcj_project::files::read_bounded(&self.path, wordsmith_manuscript_format::pack::MAX_PACKED)
                    .map_err(|e| Error::Source(format!("{}: {e}", self.path.display())))?;
                Ok(digest(&bytes) != *h)
            }
        }
    }
}

/// The words of one section in reading order: its title when it is a part or chapter, its
/// content, then its notes (not its children's).
pub fn section_words(s: &Section) -> Vec<String> {
    let mut out = Vec::new();
    if matches!(s.role, Role::Part | Role::Chapter) {
        out.extend(wordsmith_manuscript_model::words::words(&s.title));
    }
    let t = Text { blocks: s.content.clone(), notes: s.notes.clone() };
    out.extend(t.words());
    out
}

fn walk<'a>(s: &'a Section, out: &mut Vec<&'a Section>) {
    out.push(s);
    for c in &s.children {
        walk(c, out);
    }
}

/// Every section of the book in reading order (front, body with children, back).
pub fn sections(b: &Book) -> Vec<&Section> {
    let mut out = Vec::new();
    for s in b.front.iter().chain(&b.body).chain(&b.back) {
        walk(s, &mut out);
    }
    out
}

/// A checksum of each section's words, by id (hex of SHA-256 over the words, each ended by a
/// separator), for "changed since the last export".
pub fn checksums(b: &Book) -> BTreeMap<SectionId, String> {
    sections(b)
        .into_iter()
        .map(|s| {
            let mut h = Sha256::new();
            for w in section_words(s) {
                h.update(w.as_bytes());
                h.update([0x1f]);
            }
            let hex: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
            (s.id, hex)
        })
        .collect()
}

/// How a section changed between two builds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    /// New since the last build.
    Added,
    /// Gone since the last build.
    Removed,
    /// Its words changed.
    Changed,
}

/// The sections whose words changed between two sets of checksums, in id order.
pub fn changed_sections(old: &BTreeMap<SectionId, String>, new: &BTreeMap<SectionId, String>) -> Vec<(SectionId, Change)> {
    let mut out = Vec::new();
    for (id, sum) in new {
        match old.get(id) {
            None => out.push((*id, Change::Added)),
            Some(o) if o != sum => out.push((*id, Change::Changed)),
            Some(_) => {}
        }
    }
    for id in old.keys() {
        if !new.contains_key(id) {
            out.push((*id, Change::Removed));
        }
    }
    out.sort_by_key(|(id, _)| *id);
    out
}

/// Write the owned source of the project at `project_root` (create it, or save the changed
/// files of the existing one) with Manuscript's public folder writer.
pub fn write_owned(project_root: &Path, name: &str, project: &Project) -> Result<(), Error> {
    crate::check_source(&Source::Owned(name.to_owned()))?;
    let path = project_root.join(name);
    if path.join("project.toml").exists() {
        let (mut folder, _) = wordsmith_manuscript_format::folder::Folder::open(&path).map_err(source_err)?;
        folder.save(project).map_err(source_err)?;
    } else {
        wordsmith_manuscript_format::folder::Folder::create(&path, project).map_err(source_err)?;
    }
    Ok(())
}
