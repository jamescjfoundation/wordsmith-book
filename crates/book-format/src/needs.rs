// SPDX-License-Identifier: MIT OR Apache-2.0
//! What a book still needs before it can be published: the metadata the source has not set
//! (`book::missing_metadata`) and Book's own needs (a valid, unique ISBN per edition that has one,
//! a trim for each printed edition).

use wordsmith_book_model::{BookProject, Severity, isbn};
use wordsmith_manuscript_format::book::{self, Book};

/// One thing the book needs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Need {
    /// How much it matters.
    pub severity: Severity,
    /// What, in plain words.
    pub message: String,
    /// The edition it is about, if any.
    pub edition: Option<String>,
}

fn need(severity: Severity, message: String, edition: Option<&str>) -> Need {
    Need { severity, message, edition: edition.map(str::to_owned) }
}

/// Everything the book needs, most important first.
pub fn needs(project: &BookProject, b: &Book) -> Vec<Need> {
    let mut out = Vec::new();
    for missing in book::missing_metadata(b) {
        let overridden = match missing {
            "title" => project.metadata.overrides.contains_key("title"),
            "author or pen name" => project.metadata.overrides.contains_key("byline"),
            "rights statement" => project.metadata.overrides.contains_key("rights"),
            "description" => project.metadata.overrides.contains_key("description"),
            _ => false,
        };
        if !overridden {
            out.push(need(Severity::Warning, format!("The book has no {missing} yet."), None));
        }
    }
    let mut seen: Vec<(String, String)> = Vec::new();
    for e in &project.editions {
        if e.isbn.trim().is_empty() {
            out.push(need(Severity::Note, format!("The {} has no ISBN.", e.name.to_lowercase()), Some(&e.id)));
        } else {
            match isbn::check(&e.isbn) {
                Ok(d) => {
                    if let Some((other, _)) = seen.iter().find(|(_, x)| *x == d) {
                        out.push(need(Severity::Blocks, format!("The {} and the {} have the same ISBN; each edition needs its own.", other.to_lowercase(), e.name.to_lowercase()), Some(&e.id)));
                    }
                    seen.push((e.name.clone(), d));
                }
                Err(m) => out.push(need(Severity::Blocks, format!("The {}'s ISBN: {m}", e.name.to_lowercase()), Some(&e.id))),
            }
        }
        if e.format.is_print() && e.trim.trim().is_empty() {
            out.push(need(Severity::Blocks, format!("The {} has no trim size.", e.name.to_lowercase()), Some(&e.id)));
        }
    }
    if project.editions.is_empty() {
        out.push(need(Severity::Blocks, "The book has no editions to build.".into(), None));
    }
    out.sort_by_key(|n| n.severity);
    out
}
