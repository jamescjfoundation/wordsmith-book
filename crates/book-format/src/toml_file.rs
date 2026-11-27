// SPDX-License-Identifier: MIT OR Apache-2.0
//! `book.toml`: reading (with migrations from older schemas) and writing.
//!
//! Version 0 is a draft layout that was never released. It exists only so the migration path is
//! built and tested before the first real format change needs it (plan section 6.1, M1.1): its
//! source was a table `{ kind, path }` and its design a plain string.

use wordsmith_book_model::{BookProject, SCHEMA};

use crate::{Error, limits};

/// Parse TOML text that is not trusted: size and nesting first.
pub fn parse_table(text: &str, file: &str, limit: u64) -> Result<toml::Table, Error> {
    if text.len() as u64 > limit {
        return Err(Error::TooLarge { what: file.into(), limit });
    }
    if jcj_project::folder::toml_nesting(text) > limits::MAX_DEPTH {
        return Err(Error::Damaged { file: file.into(), why: format!("tables and arrays nest deeper than {}", limits::MAX_DEPTH) });
    }
    text.parse::<toml::Table>().map_err(|e| Error::Damaged { file: file.into(), why: e.message().to_owned() })
}

fn migrate_0_to_1(t: &mut toml::Table) -> Result<(), Error> {
    if let Some(toml::Value::Table(src)) = t.remove("source") {
        let kind = src.get("kind").and_then(toml::Value::as_str).unwrap_or("owned").to_owned();
        let path = src.get("path").and_then(toml::Value::as_str).unwrap_or(wordsmith_book_model::OWNED_SOURCE).to_owned();
        let mut new = toml::Table::new();
        let key = if kind == "manuscript" { "manuscript" } else { "owned" };
        new.insert(key.into(), toml::Value::String(path));
        t.insert("source".into(), toml::Value::Table(new));
    }
    if let Some(toml::Value::String(id)) = t.remove("design") {
        let mut d = toml::Table::new();
        d.insert("id".into(), toml::Value::String(id));
        t.insert("design".into(), toml::Value::Table(d));
    }
    t.insert("schema".into(), toml::Value::Integer(1));
    Ok(())
}

/// Read `book.toml`, migrating an older schema and refusing a newer one.
pub fn read(text: &str) -> Result<BookProject, Error> {
    let mut t = parse_table(text, "book.toml", limits::MAX_BOOK_TOML)?;
    let schema = match t.get("schema") {
        Some(toml::Value::Integer(n)) if *n >= 0 => u32::try_from(*n).map_err(|_| Error::NotABook)?,
        _ => return Err(Error::NotABook),
    };
    if schema > SCHEMA {
        return Err(Error::Newer { found: schema, supported: SCHEMA });
    }
    if schema == 0 {
        migrate_0_to_1(&mut t)?;
    }
    let p: BookProject = t.try_into().map_err(|e: toml::de::Error| Error::Damaged { file: "book.toml".into(), why: e.message().to_owned() })?;
    check_counts(&p)?;
    let problems = p.problems();
    if let Some(first) = problems.first() {
        return Err(Error::Damaged { file: "book.toml".into(), why: first.clone() });
    }
    crate::check_source(&p.source)?;
    Ok(p)
}

fn check_counts(p: &BookProject) -> Result<(), Error> {
    for (what, n, max) in [
        ("editions", p.editions.len(), limits::MAX_EDITIONS),
        ("elements", p.elements.len(), limits::MAX_ELEMENTS),
        ("section settings", p.sections.len(), limits::MAX_SECTIONS),
        ("fixes", p.fixes.len(), limits::MAX_FIXES),
    ] {
        if n > max {
            return Err(Error::TooLarge { what: format!("book.toml's {what}"), limit: max as u64 });
        }
    }
    Ok(())
}

/// Write `book.toml` (schema [`SCHEMA`]); the same project always gives the same text.
pub fn write(p: &BookProject) -> Result<String, Error> {
    let mut p = p.clone();
    p.schema = SCHEMA;
    let body = toml::to_string(&p).map_err(|e| Error::Write(e.to_string()))?;
    Ok(format!("# A Wordsmith Book project (docs/format.md).\n{body}"))
}
