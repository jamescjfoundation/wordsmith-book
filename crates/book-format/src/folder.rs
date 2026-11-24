// SPDX-License-Identifier: MIT OR Apache-2.0
//! The Book project folder (plan section 6.2): reading it with limits, and saving only the files
//! that changed, each atomically, `book.toml` last.
//!
//! ```text
//! My Book.wsbk/
//!   book.toml               Book's own layer (BookProject)
//!   designs/<id>.toml       the user's own designs
//!   source.wsm/             an owned source (written through Manuscript's format, not here)
//!   assets/                 cover images, pictures, fonts the user added
//!   builds/<edition>.json   the last build's record
//!   .recovery/              unsaved work while the app runs
//! ```

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use wordsmith_book_model::BookProject;

use crate::{Error, limits, toml_file};

/// The folders this module manages, besides `book.toml`.
pub const MANAGED_DIRS: [&str; 3] = ["designs", "assets", "builds"];

/// Everything in a Book project folder that Book reads and writes as files (the owned source
/// has its own format and is not here).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Files {
    /// `book.toml`.
    pub project: BookProject,
    /// `designs/<id>.toml`, by id, as text (`book-design` reads them).
    pub designs: BTreeMap<String, String>,
    /// `assets/<path>`, by path inside `assets/`.
    pub assets: BTreeMap<String, Vec<u8>>,
    /// `builds/<edition>.json`, by edition id, as text.
    pub builds: BTreeMap<String, String>,
}

/// Whether `name` is a safe file name inside the project: no folders above, no absolute path, no
/// control characters, not hidden.
pub fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= limits::MAX_NAME
        && !name.starts_with('/')
        && !name.starts_with('.')
        && !name.contains('\\')
        && !name.contains(':')
        && name.split('/').all(|seg| !seg.is_empty() && seg != "." && seg != ".." && !seg.starts_with('.'))
        && !name.chars().any(char::is_control)
}

/// Whether `id` is a safe design or edition id (a file name without folders).
pub fn safe_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 100 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

impl Files {
    /// Files for a new project.
    pub fn new(project: BookProject) -> Files {
        Files { project, ..Files::default() }
    }

    /// Every file, by its path inside the project folder, as bytes. The same files always give
    /// the same bytes.
    pub fn render(&self) -> Result<BTreeMap<String, Vec<u8>>, Error> {
        let mut out = BTreeMap::new();
        out.insert("book.toml".to_owned(), toml_file::write(&self.project)?.into_bytes());
        for (id, text) in &self.designs {
            if !safe_id(id) {
                return Err(Error::Write(format!("`{id}` is not a usable design id")));
            }
            out.insert(format!("designs/{id}.toml"), text.clone().into_bytes());
        }
        for (path, bytes) in &self.assets {
            if !safe_name(path) {
                return Err(Error::Write(format!("`{path}` is not a usable asset name")));
            }
            out.insert(format!("assets/{path}"), bytes.clone());
        }
        for (id, text) in &self.builds {
            if !safe_id(id) {
                return Err(Error::Write(format!("`{id}` is not a usable edition id")));
            }
            out.insert(format!("builds/{id}.json"), text.clone().into_bytes());
        }
        Ok(out)
    }

    /// Files from their bytes by path (a folder's or a pack's), checking every limit.
    pub fn from_map(map: &BTreeMap<String, Vec<u8>>) -> Result<Files, Error> {
        let toml = map.get("book.toml").ok_or(Error::NotABook)?;
        let text = std::str::from_utf8(toml).map_err(|_| Error::Damaged { file: "book.toml".into(), why: "not UTF-8 text".into() })?;
        let project = toml_file::read(text)?;
        let mut f = Files::new(project);
        let mut assets_total = 0u64;
        for (path, bytes) in map {
            if let Some(name) = path.strip_prefix("designs/") {
                let id = name.strip_suffix(".toml").filter(|id| safe_id(id)).ok_or_else(|| bad(path, "not a design file name"))?;
                if bytes.len() as u64 > limits::MAX_DESIGN {
                    return Err(Error::TooLarge { what: path.clone(), limit: limits::MAX_DESIGN });
                }
                let text = String::from_utf8(bytes.clone()).map_err(|_| bad(path, "not UTF-8 text"))?;
                f.designs.insert(id.to_owned(), text);
            } else if let Some(name) = path.strip_prefix("assets/") {
                if !safe_name(name) {
                    return Err(bad(path, "not a usable asset name"));
                }
                if bytes.len() as u64 > limits::MAX_ASSET {
                    return Err(Error::TooLarge { what: path.clone(), limit: limits::MAX_ASSET });
                }
                assets_total += bytes.len() as u64;
                if assets_total > limits::MAX_ASSETS_TOTAL {
                    return Err(Error::TooLarge { what: "the project's assets".into(), limit: limits::MAX_ASSETS_TOTAL });
                }
                f.assets.insert(name.to_owned(), bytes.clone());
            } else if let Some(name) = path.strip_prefix("builds/") {
                let id = name.strip_suffix(".json").filter(|id| safe_id(id)).ok_or_else(|| bad(path, "not a build record name"))?;
                if bytes.len() as u64 > limits::MAX_BUILD {
                    return Err(Error::TooLarge { what: path.clone(), limit: limits::MAX_BUILD });
                }
                f.builds.insert(id.to_owned(), String::from_utf8(bytes.clone()).map_err(|_| bad(path, "not UTF-8 text"))?);
            }
        }
        for (what, n, max) in [
            ("design files", f.designs.len(), limits::MAX_DESIGNS),
            ("assets", f.assets.len(), limits::MAX_ASSETS),
            ("build records", f.builds.len(), limits::MAX_BUILDS),
        ] {
            if n > max {
                return Err(Error::TooLarge { what: format!("the project's {what}"), limit: max as u64 });
            }
        }
        Ok(f)
    }
}

fn bad(file: &str, why: &str) -> Error {
    Error::Damaged { file: file.into(), why: why.into() }
}

fn io(file: &str, e: impl std::fmt::Display) -> Error {
    Error::Io { file: file.into(), why: e.to_string() }
}

fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

/// Read every managed file of the folder at `root` into memory, checking sizes and counts as it
/// goes (never more than the limits are read).
pub fn read_folder(root: &Path) -> Result<BTreeMap<String, Vec<u8>>, Error> {
    let mut out = BTreeMap::new();
    let toml = root.join("book.toml");
    if !toml.is_file() {
        return Err(Error::NotABook);
    }
    out.insert("book.toml".to_owned(), read_bounded(&toml, "book.toml", limits::MAX_BOOK_TOML)?);
    let mut count = 0usize;
    for dir in MANAGED_DIRS {
        let base = root.join(dir);
        if !base.is_dir() {
            continue;
        }
        let mut stack = vec![(base, String::from(dir))];
        while let Some((path, rel)) = stack.pop() {
            let entries = std::fs::read_dir(&path).map_err(|e| io(&rel, e))?;
            let mut names: Vec<(PathBuf, String)> = Vec::new();
            for e in entries {
                let e = e.map_err(|e| io(&rel, e))?;
                let name = e.file_name().to_string_lossy().into_owned();
                if name.starts_with('.') {
                    continue;
                }
                names.push((e.path(), format!("{rel}/{name}")));
            }
            names.sort();
            for (p, r) in names {
                count += 1;
                if count > limits::MAX_ASSETS + limits::MAX_DESIGNS + limits::MAX_BUILDS {
                    return Err(Error::TooLarge {
                        what: "the project's files".into(),
                        limit: (limits::MAX_ASSETS + limits::MAX_DESIGNS + limits::MAX_BUILDS) as u64,
                    });
                }
                let meta = std::fs::symlink_metadata(&p).map_err(|e| io(&r, e))?;
                if meta.is_dir() {
                    if r.matches('/').count() < 8 {
                        stack.push((p, r));
                    }
                } else if meta.is_file() {
                    let limit = if dir == "assets" { limits::MAX_ASSET } else { limits::MAX_BUILD.max(limits::MAX_DESIGN) };
                    out.insert(r.clone(), read_bounded(&p, &r, limit)?);
                }
            }
        }
    }
    Ok(out)
}

fn read_bounded(path: &Path, rel: &str, limit: u64) -> Result<Vec<u8>, Error> {
    jcj_project::files::read_bounded(path, limit).map_err(|e| {
        let msg = e.to_string();
        if msg.contains("larger") || msg.contains("too large") || msg.contains("limit") {
            Error::TooLarge { what: rel.into(), limit }
        } else {
            io(rel, msg)
        }
    })
}

/// Where saved files go: the disk, or a test's sink that can fail on purpose.
pub trait Sink {
    /// Write a file atomically.
    fn write(&mut self, rel: &str, bytes: &[u8]) -> Result<(), Error>;
    /// Remove a file.
    fn remove(&mut self, rel: &str) -> Result<(), Error>;
}

/// Writes into a project folder on disk, each file atomically.
pub struct Disk<'a>(pub &'a Path);

impl Sink for Disk<'_> {
    fn write(&mut self, rel: &str, bytes: &[u8]) -> Result<(), Error> {
        let path = self.0.join(rel);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| io(rel, e))?;
        }
        jcj_project::files::write_atomic(&path, bytes).map_err(|e| io(rel, e))
    }

    fn remove(&mut self, rel: &str) -> Result<(), Error> {
        match std::fs::remove_file(self.0.join(rel)) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(io(rel, e)),
        }
    }
}

/// What a save did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Saved {
    /// Files written, in order.
    pub written: Vec<String>,
    /// Files removed.
    pub removed: Vec<String>,
}

/// An open project folder: where it is and what its files held when last read or written.
#[derive(Clone, Debug)]
pub struct Folder {
    root: PathBuf,
    index: BTreeMap<String, [u8; 32]>,
}

impl Folder {
    /// Open the project folder at `root`.
    pub fn open(root: &Path) -> Result<(Folder, Files), Error> {
        let map = read_folder(root)?;
        let files = Files::from_map(&map)?;
        let index = map.iter().map(|(k, v)| (k.clone(), digest(v))).collect();
        Ok((Folder { root: root.to_path_buf(), index }, files))
    }

    /// Create a project folder at `root` (which must not hold a project yet) with `files`.
    pub fn create(root: &Path, files: &Files) -> Result<Folder, Error> {
        if root.join("book.toml").exists() {
            return Err(Error::Io { file: "book.toml".into(), why: "a project is already there".into() });
        }
        std::fs::create_dir_all(root).map_err(|e| io("", e))?;
        let mut f = Folder { root: root.to_path_buf(), index: BTreeMap::new() };
        f.save(files)?;
        Ok(f)
    }

    /// The project folder.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Save: write the files whose bytes changed since the last read or write, each atomically,
    /// `book.toml` last, and remove managed files that are gone. A failure leaves every file
    /// either as it was or as it is now.
    pub fn save(&mut self, files: &Files) -> Result<Saved, Error> {
        let root = self.root.clone();
        self.save_to(files, &mut Disk(&root))
    }

    /// [`Folder::save`] into any sink.
    pub fn save_to(&mut self, files: &Files, sink: &mut dyn Sink) -> Result<Saved, Error> {
        let rendered = files.render()?;
        let mut saved = Saved::default();
        let mut order: Vec<&String> = rendered.keys().filter(|k| k.as_str() != "book.toml").collect();
        order.extend(rendered.keys().filter(|k| k.as_str() == "book.toml"));
        for rel in order {
            let Some(bytes) = rendered.get(rel) else { continue };
            let d = digest(bytes);
            if self.index.get(rel) == Some(&d) {
                continue;
            }
            sink.write(rel, bytes)?;
            self.index.insert(rel.clone(), d);
            saved.written.push(rel.clone());
        }
        let gone: Vec<String> = self.index.keys().filter(|k| !rendered.contains_key(*k)).cloned().collect();
        for rel in gone {
            sink.remove(&rel)?;
            self.index.remove(&rel);
            saved.removed.push(rel);
        }
        Ok(saved)
    }
}
