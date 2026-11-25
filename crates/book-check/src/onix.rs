// SPDX-License-Identifier: MIT OR Apache-2.0
//! Distributor metadata (META-6): an ONIX for Books 3.0 message (reference tags) with one
//! product record per edition, from the book's details: its identifier (the ISBN-13, else the
//! edition's own), its form, title, series, contributors, language, page count (print), its
//! description, imprint and publisher, and its publishing date. Written in the element order the
//! published schema sets and with codes from its code lists, so the record validates against it.
//! The same book gives the same bytes.

use std::fmt::Write as _;

use wordsmith_book_model::matter::Meta;
use wordsmith_book_model::{BookProject, Edition, Format, Role};
use wordsmith_manuscript_format::book::Book;

/// The message's namespace, as the standard names it.
pub const NAMESPACE: &str = "http://ns.editeur.org/onix/3.0/reference";

/// One edition's record: the edition, and its page count when it is printed and laid out.
pub struct Record<'a> {
    /// The edition.
    pub edition: &'a Edition,
    /// Its printed page count (print editions laid out).
    pub pages: Option<usize>,
}

fn x(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// The ISO 639-2/B code the standard's language list uses, from a BCP 47 tag (`en-GB` is `eng`);
/// `und` when Book does not know it.
pub fn language_code(tag: &str) -> &'static str {
    match tag.split(['-', '_']).next().unwrap_or("").to_ascii_lowercase().as_str() {
        "en" => "eng",
        "es" => "spa",
        "fr" => "fre",
        "de" => "ger",
        "it" => "ita",
        "pt" => "por",
        "nl" => "dut",
        "sv" => "swe",
        "da" => "dan",
        "nb" | "no" => "nor",
        "fi" => "fin",
        "pl" => "pol",
        "cs" => "cze",
        "ru" => "rus",
        "el" => "gre",
        "tr" => "tur",
        "ja" => "jpn",
        "zh" => "chi",
        "ko" => "kor",
        "ar" => "ara",
        "he" => "heb",
        "hi" => "hin",
        "ca" => "cat",
        "ga" => "gle",
        "cy" => "wel",
        "la" => "lat",
        _ => "und",
    }
}

/// The contributor role code for Book's role.
fn role_code(r: Role) -> &'static str {
    match r {
        Role::Author => "A01",
        Role::Editor => "B01",
        Role::Translator => "B06",
        Role::Illustrator => "A12",
        Role::Foreword => "A23",
        Role::Introduction => "A24",
        Role::CoverDesigner => "A36",
        Role::Narrator => "E03",
    }
}

/// The product form (and detail) for an edition's format.
fn form(f: Format) -> (&'static str, Option<&'static str>) {
    match f {
        Format::Paperback | Format::LargePrint => ("BC", None),
        Format::Hardcover => ("BB", None),
        // A digital download, EPUB.
        Format::Ebook => ("ED", Some("E101")),
    }
}

fn tag(out: &mut String, depth: usize, name: &str, value: &str) {
    let _ = writeln!(out, "{}<{name}>{}</{name}>", "  ".repeat(depth), x(value));
}

fn open(out: &mut String, depth: usize, name: &str) {
    let _ = writeln!(out, "{}<{name}>", "  ".repeat(depth));
}

fn close(out: &mut String, depth: usize, name: &str) {
    let _ = writeln!(out, "{}</{name}>", "  ".repeat(depth));
}

/// The message for `records`, sent by `sender` on `date` (`YYYY-MM-DD`).
pub fn message(project: &BookProject, book: &Book, records: &[Record<'_>], sender: &str, date: &str) -> String {
    let day: String = date.chars().filter(char::is_ascii_digit).take(8).collect();
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let _ = writeln!(out, "<ONIXMessage xmlns=\"{NAMESPACE}\" release=\"3.0\">");
    open(&mut out, 1, "Header");
    open(&mut out, 2, "Sender");
    tag(&mut out, 3, "SenderName", if sender.trim().is_empty() { "Unknown" } else { sender.trim() });
    close(&mut out, 2, "Sender");
    tag(&mut out, 2, "SentDateTime", &day);
    close(&mut out, 1, "Header");
    for r in records {
        let m = wordsmith_book_model::matter::meta(&book.metadata.info, &book.metadata.byline, &project.metadata, Some(r.edition));
        product(&mut out, project, book, r, &m, &day);
    }
    out.push_str("</ONIXMessage>\n");
    out
}

fn product(out: &mut String, project: &BookProject, book: &Book, r: &Record<'_>, m: &Meta, day: &str) {
    let e = r.edition;
    let isbn = wordsmith_book_model::isbn::check(&m.isbn).ok();
    let slug: String = m.title.to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
    let slug = slug.split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-");
    open(out, 1, "Product");
    tag(out, 2, "RecordReference", &format!("book.{}.{}", if slug.is_empty() { "untitled" } else { &slug }, e.id));
    tag(out, 2, "NotificationType", "03");
    open(out, 2, "ProductIdentifier");
    match &isbn {
        Some(d) => {
            tag(out, 3, "ProductIDType", "15");
            tag(out, 3, "IDValue", d);
        }
        None => {
            // No ISBN yet: the edition's own identifier.
            tag(out, 3, "ProductIDType", "01");
            tag(out, 3, "IDTypeName", "Book edition");
            tag(out, 3, "IDValue", &format!("{}.{}", if slug.is_empty() { "untitled" } else { &slug }, e.id));
        }
    }
    close(out, 2, "ProductIdentifier");
    open(out, 2, "DescriptiveDetail");
    tag(out, 3, "ProductComposition", "00");
    let (f, detail) = form(e.format);
    tag(out, 3, "ProductForm", f);
    if let Some(d) = detail {
        tag(out, 3, "ProductFormDetail", d);
    }
    // The series: a collection, its number the part number.
    let series_name = if book.metadata.info.series.is_empty() {
        project.metadata.overrides.get("series").cloned().unwrap_or_default()
    } else {
        book.metadata.info.series.clone()
    };
    let series_name = series_name.split(", Book ").next().unwrap_or("").trim().to_owned();
    let number = project.house.series_number.or(book.metadata.info.series_number);
    if series_name.is_empty() {
        tag(out, 3, "NoCollection", "");
    } else {
        open(out, 3, "Collection");
        tag(out, 4, "CollectionType", "10");
        open(out, 4, "TitleDetail");
        tag(out, 5, "TitleType", "01");
        open(out, 5, "TitleElement");
        tag(out, 6, "TitleElementLevel", "02");
        if let Some(n) = number {
            tag(out, 6, "PartNumber", &n.to_string());
        }
        tag(out, 6, "TitleText", &series_name);
        close(out, 5, "TitleElement");
        close(out, 4, "TitleDetail");
        close(out, 3, "Collection");
    }
    open(out, 3, "TitleDetail");
    tag(out, 4, "TitleType", "01");
    open(out, 4, "TitleElement");
    tag(out, 5, "TitleElementLevel", "01");
    tag(out, 5, "TitleText", &m.title);
    if !m.subtitle.trim().is_empty() {
        tag(out, 5, "Subtitle", &m.subtitle);
    }
    close(out, 4, "TitleElement");
    close(out, 3, "TitleDetail");
    let mut seq = 0;
    let mut contributor = |out: &mut String, name: &str, role: &str| {
        seq += 1;
        open(out, 3, "Contributor");
        tag(out, 4, "SequenceNumber", &seq.to_string());
        tag(out, 4, "ContributorRole", role);
        tag(out, 4, "PersonName", name);
        close(out, 3, "Contributor");
    };
    if !m.byline.trim().is_empty() {
        contributor(out, m.byline.trim(), "A01");
    }
    for c in &project.metadata.contributors {
        if !c.name.trim().is_empty() {
            contributor(out, c.name.trim(), role_code(c.role));
        }
    }
    if seq == 0 {
        tag(out, 3, "NoContributor", "");
    }
    open(out, 3, "Language");
    tag(out, 4, "LanguageRole", "01");
    tag(out, 4, "LanguageCode", language_code(&m.language));
    close(out, 3, "Language");
    if let (true, Some(pages)) = (e.format.is_print(), r.pages) {
        open(out, 3, "Extent");
        tag(out, 4, "ExtentType", "00");
        tag(out, 4, "ExtentValue", &pages.to_string());
        tag(out, 4, "ExtentUnit", "03");
        close(out, 3, "Extent");
    }
    for code in project.metadata.subject_codes.iter().filter(|c| !c.trim().is_empty()) {
        open(out, 3, "Subject");
        tag(out, 4, "SubjectSchemeIdentifier", "20");
        tag(out, 4, "SubjectHeadingText", code.trim());
        close(out, 3, "Subject");
    }
    close(out, 2, "DescriptiveDetail");
    let description = project.metadata.overrides.get("description").cloned().unwrap_or_default();
    if !description.trim().is_empty() {
        open(out, 2, "CollateralDetail");
        open(out, 3, "TextContent");
        tag(out, 4, "TextType", "03");
        tag(out, 4, "ContentAudience", "00");
        tag(out, 4, "Text", description.trim());
        close(out, 3, "TextContent");
        close(out, 2, "CollateralDetail");
    }
    open(out, 2, "PublishingDetail");
    if !m.imprint.trim().is_empty() {
        open(out, 3, "Imprint");
        tag(out, 4, "ImprintName", m.imprint.trim());
        close(out, 3, "Imprint");
    }
    // The schema wants a publisher: the imprint, else the author (published by them).
    let publisher = [m.imprint.trim(), m.byline.trim()].into_iter().find(|n| !n.is_empty()).unwrap_or("Unknown");
    open(out, 3, "Publisher");
    tag(out, 4, "PublishingRole", "01");
    tag(out, 4, "PublisherName", publisher);
    close(out, 3, "Publisher");
    tag(out, 3, "PublishingStatus", "04");
    let year = m.year.trim();
    if year.len() == 4 && year.chars().all(|c| c.is_ascii_digit()) {
        open(out, 3, "PublishingDate");
        tag(out, 4, "PublishingDateRole", "01");
        let _ = writeln!(out, "        <Date dateformat=\"05\">{year}</Date>");
        close(out, 3, "PublishingDate");
    } else if day.len() == 8 {
        open(out, 3, "PublishingDate");
        tag(out, 4, "PublishingDateRole", "01");
        tag(out, 4, "Date", day);
        close(out, 3, "PublishingDate");
    }
    close(out, 2, "PublishingDetail");
    close(out, 1, "Product");
}
