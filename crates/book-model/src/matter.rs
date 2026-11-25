// SPDX-License-Identifier: MIT OR Apache-2.0
//! Front and back matter made from the book's metadata (STR-4 to STR-6): the half title, title
//! page, copyright page, contents, "also by" and "about the author", each from a template the user
//! can edit. Templates fill placeholders in braces (`{title}`, `{byline}`, `{year}`, ...); an
//! unknown placeholder is left as written and reported.

use wordsmith_manuscript_model::{Block, BookInfo, Inline, Marks};

use crate::{Edition, ElementKind, Metadata};

/// The book's metadata as front and back matter use it: the source's, with Book's additions and
/// the edition's.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Meta {
    /// The title.
    pub title: String,
    /// The subtitle.
    pub subtitle: String,
    /// The name the book is published under.
    pub byline: String,
    /// The copyright year.
    pub year: String,
    /// The copyright holder.
    pub holder: String,
    /// The rights statement.
    pub rights: String,
    /// The imprint's name.
    pub imprint: String,
    /// The imprint's address lines.
    pub address: Vec<String>,
    /// The imprint's website.
    pub website: String,
    /// The imprint's logo, a picture path (`assets/…`; empty: none).
    pub logo: String,
    /// The imprint's copyright wording (empty: Book's).
    pub copyright: String,
    /// The edition's ISBN.
    pub isbn: String,
    /// The edition's format, as words ("paperback").
    pub format: String,
    /// The series and number ("The Lighthouse Books, Book 2").
    pub series: String,
    /// The language (BCP 47).
    pub language: String,
    /// Contributors besides the author, as lines ("Translated by A. Name").
    pub credits: Vec<String>,
    /// The author's other books, grouped by series: (series, titles).
    pub also_by: Vec<(String, Vec<String>)>,
    /// The author's biography.
    pub about: String,
}

/// The metadata for `edition` (or the book as a whole): the source's `info` and `byline`, then
/// Book's additions over them.
pub fn meta(info: &BookInfo, byline: &str, add: &Metadata, edition: Option<&Edition>) -> Meta {
    let pick = |key: &str, from: &str| add.overrides.get(key).cloned().unwrap_or_else(|| from.to_owned());
    let byline = pick("byline", byline);
    let imprint = add.imprint.clone().unwrap_or_default();
    let isbn = edition.map(|e| e.isbn.clone()).filter(|s| !s.is_empty()).or_else(|| {
        let key = if edition.is_some_and(|e| e.format == crate::Format::Ebook) { "ebook" } else { "print" };
        info.isbn.get(key).cloned()
    });
    let mut also: Vec<(String, Vec<String>)> = Vec::new();
    for b in &add.also_by {
        match also.iter_mut().find(|(s, _)| *s == b.series) {
            Some((_, titles)) => titles.push(b.title.clone()),
            None => also.push((b.series.clone(), vec![b.title.clone()])),
        }
    }
    Meta {
        title: pick("title", &info.title),
        subtitle: pick("subtitle", &info.subtitle),
        holder: if add.copyright_holder.is_empty() { byline.clone() } else { add.copyright_holder.clone() },
        byline,
        year: info.copyright_year.map(|y| y.to_string()).unwrap_or_default(),
        rights: pick("rights", &info.rights),
        imprint: if imprint.name.is_empty() { info.publisher.clone() } else { imprint.name },
        address: imprint.address,
        website: imprint.website,
        logo: imprint.logo,
        copyright: imprint.copyright,
        isbn: isbn.unwrap_or_default(),
        format: edition.map(|e| e.format.title().to_lowercase()).unwrap_or_default(),
        // A house series the book joined stands in for the source's (META-4).
        series: match (info.series.is_empty(), info.series_number) {
            _ if add.overrides.get("series").is_some_and(|s| !s.trim().is_empty()) => add.overrides.get("series").cloned().unwrap_or_default(),
            (true, _) => String::new(),
            (false, Some(n)) => format!("{}, Book {n}", info.series),
            (false, None) => info.series.clone(),
        },
        language: info.language.clone(),
        credits: add.contributors.iter().filter(|c| c.role != crate::Role::Author).map(|c| format!("{} {}", credit_word(c.role), c.name)).collect(),
        also_by: also,
        about: pick("about", &add.about_author),
    }
}

fn credit_word(r: crate::Role) -> &'static str {
    match r {
        crate::Role::Author => "By",
        crate::Role::Editor => "Edited by",
        crate::Role::Translator => "Translated by",
        crate::Role::Illustrator => "Illustrated by",
        crate::Role::Foreword => "With a foreword by",
        crate::Role::Introduction => "With an introduction by",
        crate::Role::CoverDesigner => "Cover by",
        crate::Role::Narrator => "Read by",
    }
}

/// Marks a filled `{logo}`: the line is the logo picture's path.
const LOGO: char = '\u{E030}';

/// The default template of an element kind, or none when it is made another way (contents, also
/// by) or written by the author (dedication, epigraph, acknowledgements, ...).
pub fn default_template(kind: ElementKind) -> Option<&'static str> {
    match kind {
        ElementKind::HalfTitle => Some("{title}"),
        ElementKind::TitlePage => Some("{title}\n{subtitle}\n{byline}\n{credits}\n{imprint}\n{logo}"),
        ElementKind::Copyright => Some("Copyright © {year} {holder}\n{rights}\n{isbn_line}\n{series}\nPublished by {imprint}\n{address}\n{website}"),
        ElementKind::AboutTheAuthor => Some("{about}"),
        ElementKind::Newsletter => Some("To hear about new books first, sign up at {website}."),
        _ => None,
    }
}

/// One generated paragraph and how it is set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    /// Its text.
    pub text: String,
    /// What it is ("title", "subtitle", "byline", "small", "heading", "entry", "body").
    pub style: &'static str,
}

fn value(m: &Meta, key: &str) -> Option<String> {
    Some(match key {
        "title" => m.title.clone(),
        "subtitle" => m.subtitle.clone(),
        "byline" => m.byline.clone(),
        "year" => m.year.clone(),
        "holder" => m.holder.clone(),
        "rights" => m.rights.clone(),
        "imprint" => m.imprint.clone(),
        "address" => m.address.join("\n"),
        "website" => m.website.clone(),
        // The logo is a picture on a line of its own (see `lines`).
        "logo" => {
            if m.logo.is_empty() {
                String::new()
            } else {
                format!("{LOGO}{}", m.logo)
            }
        }
        "isbn" => m.isbn.clone(),
        "isbn_line" => {
            if m.isbn.is_empty() {
                String::new()
            } else if m.format.is_empty() {
                format!("ISBN {}", m.isbn)
            } else {
                format!("ISBN {} ({})", m.isbn, m.format)
            }
        }
        "format" => m.format.clone(),
        "series" => m.series.clone(),
        "language" => m.language.clone(),
        "credits" => m.credits.join("\n"),
        "about" => m.about.clone(),
        _ => return None,
    })
}

/// Fill a template's placeholders. Gives the text and the placeholders it did not know.
pub fn fill(template: &str, m: &Meta) -> (String, Vec<String>) {
    let mut out = String::with_capacity(template.len());
    let mut unknown = Vec::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find('}') {
            Some(close) if after[..close].chars().all(|c| c.is_ascii_alphanumeric() || c == '_') && close > 0 => {
                let key = &after[..close];
                match value(m, key) {
                    Some(v) => out.push_str(&v),
                    None => {
                        unknown.push(key.to_owned());
                        out.push_str(&rest[open..open + close + 2]);
                    }
                }
                rest = &after[close + 1..];
            }
            _ => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    (out, unknown)
}

fn styled(kind: ElementKind, i: usize) -> &'static str {
    match (kind, i) {
        (ElementKind::HalfTitle, _) | (ElementKind::TitlePage, 0) => "title",
        (ElementKind::TitlePage, 1) => "subtitle",
        (ElementKind::TitlePage, 2) => "byline",
        (ElementKind::TitlePage, _) | (ElementKind::Copyright, _) => "small",
        (ElementKind::Glossary, _) => "term",
        (ElementKind::Questions, _) => "question",
        _ => "body",
    }
}

/// The lines an element shows: its own text (a template) or its kind's default, filled from the
/// metadata. Empty lines are left out. Gives the lines and the unknown placeholders.
pub fn lines(kind: ElementKind, text: &str, m: &Meta) -> (Vec<Line>, Vec<String>) {
    if kind == ElementKind::AlsoBy && text.is_empty() {
        let mut out = Vec::new();
        for (series, titles) in &m.also_by {
            if !series.is_empty() {
                out.push(Line { text: series.clone(), style: "heading" });
            }
            out.extend(titles.iter().map(|t| Line { text: t.clone(), style: "entry" }));
        }
        return (out, Vec::new());
    }
    // The imprint's own copyright wording stands in for Book's (META-3).
    let template = match (text.is_empty(), kind) {
        (true, ElementKind::Copyright) if !m.copyright.trim().is_empty() => m.copyright.as_str(),
        (true, _) => default_template(kind).unwrap_or(""),
        (false, _) => text,
    };
    let (filled, unknown) = fill(template, m);
    let out = filled
        .split('\n')
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .enumerate()
        .map(|(i, l)| match l.strip_prefix(LOGO) {
            Some(path) => Line { text: path.to_owned(), style: "logo" },
            None => Line { text: l.to_owned(), style: styled(kind, i) },
        })
        .collect();
    (out, unknown)
}

/// The lines as blocks of semantic text (headings for titles, paragraphs for the rest).
///
/// A glossary's lines are terms: “term: definition” or “term — definition”, the term in small
/// capitals. A list of questions is numbered.
pub fn blocks(lines: &[Line]) -> Vec<Block> {
    let mut question = 0;
    lines
        .iter()
        .map(|l| match l.style {
            "title" => Block::Heading { level: 1, content: vec![Inline::plain(&l.text)] },
            "subtitle" | "heading" => Block::Heading { level: 2, content: vec![Inline::plain(&l.text)] },
            "byline" => Block::Heading { level: 3, content: vec![Inline::plain(&l.text)] },
            "term" => Block::Para(term(&l.text)),
            // The imprint's logo: a picture, decorative (the imprint's name is on the page).
            "logo" => Block::Image { alt: String::new(), src: l.text.clone(), role: wordsmith_manuscript_model::ImageRole::Inline },
            "question" => {
                question += 1;
                let text = l.text.trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == ')').trim_start();
                Block::Para(vec![Inline::marked(format!("{question}. {text}"), Marks::default())])
            }
            _ => Block::Para(vec![Inline::marked(&l.text, Marks::default())]),
        })
        .collect()
}

/// A glossary line: the term in small capitals, then the rest as written.
fn term(line: &str) -> Vec<Inline> {
    let cut = line.find(" — ").or_else(|| line.find(':'));
    match cut {
        Some(i) if i > 0 => vec![Inline::marked(&line[..i], Marks { small_caps: true, ..Marks::default() }), Inline::plain(&line[i..])],
        _ => vec![Inline::plain(line)],
    }
}

/// The elements a new book starts with: half title, title page, copyright and contents in front,
/// about the author and "also by" at the back.
pub fn default_elements() -> Vec<crate::Element> {
    [ElementKind::HalfTitle, ElementKind::TitlePage, ElementKind::Copyright, ElementKind::Contents, ElementKind::AboutTheAuthor, ElementKind::AlsoBy]
        .into_iter()
        .map(|k| crate::Element {
            id: serde_json::to_value(k).ok().and_then(|v| v.as_str().map(str::to_owned)).unwrap_or_default(),
            kind: Some(k),
            placement: k.placement(),
            ..crate::Element::default()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_fill_and_report_unknown_placeholders() {
        let m = Meta { title: "The Lighthouse".into(), year: "2026".into(), holder: "A. Writer".into(), ..Meta::default() };
        assert_eq!(fill("Copyright © {year} {holder}. {nope} {", &m), ("Copyright © 2026 A. Writer. {nope} {".into(), vec!["nope".into()]));
        let (l, _) = lines(ElementKind::HalfTitle, "", &m);
        assert_eq!(l, vec![Line { text: "The Lighthouse".into(), style: "title" }]);
    }
}
