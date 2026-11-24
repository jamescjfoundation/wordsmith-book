// SPDX-License-Identifier: MIT OR Apache-2.0
//! Export (EXP-1, EXP-2, SPEC section 9): each edition built, preflighted and word-checked in
//! memory; files are written only when nothing blocks export, and then all at once, so a failed
//! check writes nothing. The same project with the same app version gives the same bytes.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use wordsmith_book_format::source::Opened;
use wordsmith_book_model::BookProject;

use crate::preflight::{self, Report};
use crate::validator;

/// One built file, checked and ready to write.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Built {
    /// The edition it is (its id).
    pub edition: String,
    /// Its file name.
    pub name: String,
    /// Its bytes.
    pub bytes: Vec<u8>,
    /// Its preflight report.
    pub report: Report,
}

/// Why nothing was written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// An edition's preflight found problems that block export (its report).
    Blocked(Box<Report>),
    /// An edition could not be built.
    Build(String, String),
    /// A file could not be written.
    Write(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Blocked(r) => {
                let n = r.of(wordsmith_book_model::Severity::Blocks).len();
                write!(
                    f,
                    "nothing was exported: the {} has {n} problem{} that block export (see Preflight)",
                    r.edition,
                    if n == 1 { "" } else { "s" }
                )
            }
            Error::Build(e, why) => write!(f, "nothing was exported: the {e} could not be built: {why}"),
            Error::Write(why) => write!(f, "the files could not be written: {why}"),
        }
    }
}

impl std::error::Error for Error {}

/// A name safe as a file name on every platform: letters, digits, spaces and a few marks.
pub fn file_name(s: &str) -> String {
    let clean: String =
        s.chars().map(|c| if c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.' | ',' | '\'' | '’') { c } else { ' ' }).collect();
    let clean = clean.split_whitespace().collect::<Vec<_>>().join(" ");
    let clean = clean.trim_matches('.').to_owned();
    if clean.is_empty() { "Book".into() } else { clean.chars().take(120).collect() }
}

/// Build and check one edition. `images` gives a picture's bytes by its path; `validator`, when
/// given, runs on an ebook.
pub fn build(
    project: &BookProject,
    designs: &BTreeMap<String, String>,
    source: &Opened,
    edition_id: &str,
    images: &dyn Fn(&str) -> Option<Vec<u8>>,
    validator: Option<&validator::Settings>,
) -> Result<Built, Error> {
    let edition =
        project.editions.iter().find(|e| e.id == edition_id).ok_or_else(|| Error::Build(edition_id.to_owned(), "there is no such edition".into()))?;
    let (mut report, bytes) = preflight::preflight(project, designs, source, edition, images);
    if let (Some(v), Some(b), false) = (validator, &bytes, edition.format.is_print()) {
        preflight::add_validator(&mut report, b, v);
    }
    if report.blocked() {
        return Err(Error::Blocked(Box::new(report)));
    }
    let bytes = bytes.ok_or_else(|| Error::Build(edition.name.to_lowercase(), "nothing was built".into()))?;
    let book = wordsmith_book_format::fixes::book_for_build(&project.fixes, source);
    let title = project.metadata.overrides.get("title").cloned().unwrap_or_else(|| book.metadata.info.title.clone());
    let ext = if edition.format.is_print() { "pdf" } else { "epub" };
    Ok(Built { edition: edition.id.clone(), name: format!("{} - {}.{ext}", file_name(&title), file_name(&edition.name)), bytes, report })
}

/// Build and check every edition named; all or nothing.
pub fn build_all(
    project: &BookProject,
    designs: &BTreeMap<String, String>,
    source: &Opened,
    editions: &[String],
    images: &dyn Fn(&str) -> Option<Vec<u8>>,
    validator: Option<&validator::Settings>,
) -> Result<Vec<Built>, Error> {
    editions.iter().map(|e| build(project, designs, source, e, images, validator)).collect()
}

/// Write checked files into `folder` (made if missing), each atomically; gives their paths.
/// Files already checked are the only ones this writes: a failed check never reaches here.
pub fn write(folder: &Path, files: &[(String, Vec<u8>)]) -> Result<Vec<PathBuf>, Error> {
    std::fs::create_dir_all(folder).map_err(|e| Error::Write(e.to_string()))?;
    let mut out = Vec::new();
    for (name, bytes) in files {
        if name.contains(['/', '\\']) || name.starts_with('.') {
            return Err(Error::Write(format!("“{name}” is not a file name")));
        }
        let path = folder.join(name);
        jcj_project::files::write_atomic(&path, bytes).map_err(|e| Error::Write(e.to_string()))?;
        out.push(path);
    }
    Ok(out)
}
