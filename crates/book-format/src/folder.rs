// SPDX-License-Identifier: MIT OR Apache-2.0
//! The Book project folder (plan section 6.2): what its files hold, and Book's names over the
//! core's folder layer (`jcj_project::folder`, core decision 0018), which reads the folder with
//! limits and saves only the files that changed, each atomically, `book.toml` last.
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
//!
//! What is Book's here is the layout ([`FORMAT`]), what each file holds ([`Files`]) and the
//! limits on them; the reading, the saving, the atomic writes and the sinks are the core's.

use std::collections::BTreeMap;
use std::path::Path;

use jcj_project::folder as core;
pub use jcj_project::folder::{Disk, Saved, Sink};
use wordsmith_book_model::BookProject;

use crate::{Error, limits, toml_file};

/// The folders Book reads into [`Files`], besides `book.toml`.
pub const MANAGED_DIRS: [&str; 3] = ["designs", "assets", "builds"];

/// Whether a path is one of a Book project's files: no part of it hidden, no control character
/// (the core has made sure it is a plain path already).
fn belongs(path: &str) -> bool {
    path.split('/').all(|part| !part.starts_with('.')) && !path.chars().any(char::is_control)
}

/// The Book project folder's layout, for the core's folder layer: `book.toml` at the schema of
/// Book's model, written last alone; any path that is not hidden; the packed form's type and
/// extension; a linked source's packed copy and pictures stored in a packed project as they
/// are; no conflict copies; `book.toml` at most [`limits::MAX_BOOK_TOML`].
pub const FORMAT: core::Format = core::Format {
    belongs,
    stored: &[".wsmz", ".png", ".jpg"],
    limits: core::Limits { manifest: limits::MAX_BOOK_TOML, ..core::Limits::DEFAULT },
    ..core::Format::new("book.toml", wordsmith_book_model::SCHEMA, crate::pack::MIMETYPE, crate::PACKED_EXTENSION)
};

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
    core::plain_path(name, limits::MAX_NAME) && belongs(name)
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

/// An open project folder: the core's folder, which remembers what each file held when it was
/// last read or written.
#[derive(Clone, Debug)]
pub struct Folder(core::Folder);

impl Folder {
    /// Open the project folder at `root`: `book.toml`, then every file under the folders Book
    /// keeps ([`MANAGED_DIRS`]), each within its limit.
    pub fn open(root: &Path) -> Result<(Folder, Files), Error> {
        let mut folder = core::Folder::open(root, &FORMAT).map_err(|e| Error::from_core(e, "book.toml"))?;
        let mut map = BTreeMap::new();
        let toml = core::Source::read(&mut folder, FORMAT.manifest, FORMAT.limits.manifest).map_err(|e| Error::from_core(e, "book.toml"))?;
        map.insert(FORMAT.manifest.to_owned(), toml.ok_or(Error::NotABook)?);
        for dir in MANAGED_DIRS {
            let limit = if dir == "assets" { limits::MAX_ASSET } else { limits::MAX_BUILD.max(limits::MAX_DESIGN) };
            map.extend(folder.read_tree(dir, limit).map_err(|e| Error::from_core(e, dir))?);
        }
        let files = Files::from_map(&map)?;
        Ok((Folder(folder), files))
    }

    /// Create a project folder at `root` (which must not hold a project yet) with `files`.
    pub fn create(root: &Path, files: &Files) -> Result<Folder, Error> {
        core::Folder::create(root, &FORMAT, &files.render()?).map(Folder).map_err(|e| Error::from_core(e, "book.toml"))
    }

    /// The project folder.
    pub fn root(&self) -> &Path {
        self.0.root()
    }

    /// Save: write the files whose bytes changed since the last read or write, each atomically,
    /// `book.toml` last, and remove managed files that are gone. A failure leaves every file
    /// either as it was or as it is now.
    pub fn save(&mut self, files: &Files) -> Result<Saved, Error> {
        self.0.save(&files.render()?).map_err(|e| Error::from_core(e, "the project"))
    }

    /// [`Folder::save`] into any sink.
    pub fn save_to(&mut self, files: &Files, sink: &mut dyn Sink) -> Result<Saved, Error> {
        self.0.save_to(&files.render()?, sink).map_err(|e| Error::from_core(e, "the project"))
    }

    /// [`Folder::save`] with recovery around it ([`crate::pack::save_safely`]).
    pub(crate) fn save_safely(&mut self, files: &Files) -> Result<Saved, Error> {
        self.0.save_safely(&files.render()?).map_err(|e| Error::from_core(e, "the project"))
    }
}
