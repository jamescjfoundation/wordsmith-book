// SPDX-License-Identifier: MIT OR Apache-2.0
//! EPUB import (IMP-9): an EPUB's chapters read through the core's reader, with the EPUB's own
//! structure taken from its package and documents. The navigation document (the ebook's own
//! contents) is left out, as Book makes the contents again; each document the package marks as a
//! part, chapter, front or back matter (`epub:type`, or the role it plays in the spine) starts a
//! section of that kind at its first heading; a number line ("Part One", "Chapter 3", "12")
//! before a document's title is the number Book's numbering makes again and is left out. Every
//! other word is kept, and the import's word check compares against the EPUB's text less those.
//!
//! Bounded: the package and documents are read one by one, each capped in size; a package Book
//! cannot map onto the reader's chapters is imported as a plain document.

use std::io::Read as _;
use std::sync::Arc;

use jcj_doc::{Block as DocBlock, Document};

use crate::detect::{self, Kind, Mark};
use crate::{Error, ImportRules, Imported, Report};

/// The largest file read from the package, in bytes.
const MAX_ENTRY: u64 = 16 << 20;
/// The most bytes read from one package in all.
const MAX_TOTAL: u64 = 256 << 20;
/// The most documents looked at (as the core's reader).
const MAX_DOCS: usize = 10_000;

/// What the package says about one of its reading-order documents.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DocRole {
    /// It is the navigation document (the contents).
    pub nav: bool,
    /// The kind of section it is, when the package says.
    pub kind: Option<Kind>,
}

fn entry(zip: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>, name: &str) -> Option<String> {
    let f = zip.by_name(name).ok()?;
    let mut b = Vec::new();
    f.take(MAX_ENTRY + 1).read_to_end(&mut b).ok()?;
    if b.len() as u64 > MAX_ENTRY {
        return None;
    }
    String::from_utf8(b).ok()
}

fn parse(text: &str) -> Option<roxmltree::Document<'_>> {
    roxmltree::Document::parse_with_options(text, roxmltree::ParsingOptions { allow_dtd: true, nodes_limit: 4_000_000, ..Default::default() }).ok()
}

fn join(base: &str, href: &str) -> String {
    let href = href.split('#').next().unwrap_or(href);
    let mut parts: Vec<&str> = base.rsplit_once('/').map_or(Vec::new(), |(d, _)| d.split('/').collect());
    for seg in href.split('/') {
        match seg {
            "." | "" => {}
            ".." => {
                parts.pop();
            }
            s => parts.push(s),
        }
    }
    parts.join("/")
}

const EPUB_NS: &str = "http://www.idpf.org/2007/ops";

/// The kind a document's `epub:type` (on its body or first section) names.
fn kind_of(types: &str) -> (bool, Option<Kind>) {
    let t: Vec<&str> = types.split_whitespace().collect();
    let has = |w: &str| t.contains(&w);
    if has("toc") || has("landmarks") || has("page-list") {
        return (true, None);
    }
    let kind = if has("part") || has("volume") {
        Some(Kind::Part)
    } else if has("chapter") || has("prologue") || has("epilogue") {
        Some(Kind::Chapter)
    } else if has("frontmatter")
        || has("titlepage")
        || has("halftitlepage")
        || has("copyright-page")
        || has("dedication")
        || has("epigraph")
        || has("foreword")
        || has("preface")
        || has("introduction")
    {
        Some(Kind::Front)
    } else if has("backmatter")
        || has("afterword")
        || has("acknowledgments")
        || has("appendix")
        || has("colophon")
        || has("endnotes")
        || has("glossary")
    {
        Some(Kind::Back)
    } else if has("bodymatter") {
        Some(Kind::Chapter)
    } else {
        None
    };
    (false, kind)
}

/// What the package says about each reading-order document the core's reader reads, in order;
/// `None` when the package cannot be read.
pub fn roles(bytes: &[u8]) -> Option<Vec<DocRole>> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).ok()?;
    let container = entry(&mut zip, "META-INF/container.xml")?;
    let c = parse(&container)?;
    let opf_path = c.descendants().find(|n| n.has_tag_name("rootfile")).and_then(|n| n.attribute("full-path"))?.to_owned();
    let opf_text = entry(&mut zip, &opf_path)?;
    let opf = parse(&opf_text)?;
    let manifest: std::collections::BTreeMap<&str, (&str, &str, &str)> = opf
        .descendants()
        .filter(|n| n.has_tag_name("item"))
        .filter_map(|n| {
            Some((n.attribute("id")?, (n.attribute("href")?, n.attribute("media-type").unwrap_or(""), n.attribute("properties").unwrap_or(""))))
        })
        .collect();
    let mut out = Vec::new();
    for itemref in opf.descendants().filter(|n| n.has_tag_name("itemref")).filter(|n| n.attribute("linear") != Some("no")) {
        let Some(&(href, media, props)) = itemref.attribute("idref").and_then(|i| manifest.get(i)) else { continue };
        // The core's reader reads only (X)HTML documents.
        if !(media.contains("html") || href.ends_with(".xhtml") || href.ends_with(".html") || href.ends_with(".htm")) {
            continue;
        }
        if out.len() >= MAX_DOCS {
            break;
        }
        let path = join(&opf_path, href);
        let mut role = DocRole { nav: props.split_whitespace().any(|p| p == "nav"), kind: None };
        if let Some(text) = entry(&mut zip, &path)
            && let Some(doc) = parse(&text)
        {
            let types: String = doc
                .descendants()
                .filter(|n| n.has_tag_name("body") || n.has_tag_name("section") || n.has_tag_name("nav"))
                .take(3)
                .filter_map(|n| n.attribute((EPUB_NS, "type")))
                .collect::<Vec<_>>()
                .join(" ");
            let (nav, kind) = kind_of(&types);
            role.nav |= nav;
            role.kind = kind;
        }
        out.push(role);
    }
    Some(out)
}

/// The reader's paragraphs, split into the documents they came from: each document after the
/// first starts with a page break the reader put there. `None` when the count does not match.
fn chunks(doc: &Document, docs: usize) -> Option<Vec<std::ops::Range<usize>>> {
    let mut starts = vec![0];
    for (i, b) in doc.body.iter().enumerate().skip(1) {
        if b.as_para().is_some_and(|p| p.props.page_break_before == Some(true)) {
            starts.push(i);
        }
    }
    if starts.len() != docs || doc.body.is_empty() {
        return None;
    }
    let mut out = Vec::with_capacity(docs);
    for (k, s) in starts.iter().enumerate() {
        out.push(*s..starts.get(k + 1).copied().unwrap_or(doc.body.len()));
    }
    Some(out)
}

/// Whether a line is only a section number: "Part One", "PART II", "Chapter 12", "BOOK THE FIRST",
/// "12", "XII".
fn number_line(s: &str) -> bool {
    let t = s.trim().trim_end_matches(['.', ':']);
    if t.is_empty() || t.split_whitespace().count() > 3 {
        return false;
    }
    if detect::is_number(t) {
        return true;
    }
    let mut w = t.split_whitespace();
    let first = w.next().unwrap_or("").to_lowercase();
    let rest: Vec<&str> = w.collect();
    matches!(first.as_str(), "part" | "chapter" | "book" | "volume")
        && !rest.is_empty()
        && rest.iter().all(|x| detect::is_number(x) || x.chars().all(char::is_alphabetic))
}

/// Marks a note reference's place in the text the core's reader reads: start, the note's id, end.
const REF_START: char = '\u{E010}';
const REF_END: char = '\u{E011}';
/// Marks a line the EPUB itself calls a label (`class="label"`: a number its numbering made).
const LABEL: char = '\u{E012}';
/// The most notes kept from one EPUB.
const MAX_NOTES: usize = 20_000;

fn has_type(n: &roxmltree::Node<'_, '_>, words: &[&str]) -> bool {
    let t = n.attribute((EPUB_NS, "type")).unwrap_or("");
    let r = n.attribute("role").unwrap_or("");
    t.split_whitespace().chain(r.split_whitespace()).any(|w| words.contains(&w.trim_start_matches("doc-")))
}

/// A note's paragraphs as text, its back links left out.
fn note_text(n: roxmltree::Node<'_, '_>) -> Vec<String> {
    fn text(n: roxmltree::Node<'_, '_>, out: &mut String) {
        for c in n.children() {
            if c.is_text() {
                out.push_str(c.text().unwrap_or(""));
            } else if c.is_element() && !has_type(&c, &["backlink"]) {
                text(c, out);
            }
        }
    }
    let paras: Vec<roxmltree::Node<'_, '_>> = n.descendants().filter(|d| d.has_tag_name("p")).collect();
    let mut out = Vec::new();
    for p in if paras.is_empty() { vec![n] } else { paras } {
        let mut s = String::new();
        text(p, &mut s);
        let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
        if !s.is_empty() {
            out.push(s);
        }
    }
    out
}

/// One reading-order document with its note references made into marks and its notes taken out:
/// the new text and the notes found, by id.
fn take_notes(xhtml: &str, notes: &mut std::collections::BTreeMap<String, Vec<String>>) -> Option<String> {
    let doc = parse(xhtml)?;
    let mut cuts: Vec<(std::ops::Range<usize>, String)> = Vec::new();
    for n in doc.descendants().filter(|n| n.is_element()) {
        if n.attribute("class").is_some_and(|c| c.split_whitespace().any(|w| w == "label"))
            && matches!(n.tag_name().name(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "div" | "span")
        {
            // The same element, its text marked: insert the mark after its start tag.
            let r = n.range();
            let el = xhtml.get(r.clone())?;
            let open = el.find('>')? + 1;
            cuts.push((r, format!("{}{LABEL}{}", &el[..open], &el[open..])));
        } else if n.has_tag_name("a") && has_type(&n, &["noteref"]) {
            let id = n.attribute("href").and_then(|h| h.split_once('#')).map(|(_, i)| i.to_owned()).unwrap_or_default();
            cuts.push((n.range(), format!("{REF_START}{id}{REF_END}")));
        } else if has_type(&n, &["footnote", "endnote", "rearnote", "note"])
            && !n.has_tag_name("a")
            && !n.has_tag_name("section")
            && !n.has_tag_name("ol")
        {
            if let Some(id) = n.attribute("id")
                && notes.len() < MAX_NOTES
            {
                notes.insert(id.to_owned(), note_text(n));
            }
            cuts.push((n.range(), String::new()));
        }
    }
    // Outermost cuts only, in order.
    cuts.sort_by_key(|c| c.0.start);
    let mut out = String::with_capacity(xhtml.len());
    let mut at = 0;
    for (r, with) in cuts {
        if r.start < at {
            continue;
        }
        out.push_str(xhtml.get(at..r.start)?);
        out.push_str(&with);
        at = r.end;
    }
    out.push_str(xhtml.get(at..)?);
    Some(out)
}

/// The EPUB with its reading-order documents' notes taken out and labels marked (see
/// [`take_notes`]); `None` when it has neither or cannot be read.
fn without_notes(bytes: &[u8]) -> Option<(Vec<u8>, std::collections::BTreeMap<String, Vec<String>>)> {
    use std::io::Write as _;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).ok()?;
    if zip.len() > 20_000 {
        return None;
    }
    let mut notes = std::collections::BTreeMap::new();
    let mut files: Vec<(String, Vec<u8>)> = Vec::with_capacity(zip.len());
    let mut total = 0u64;
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).ok()?;
        let name = f.name().to_owned();
        let mut b = Vec::new();
        (&mut f).take(MAX_ENTRY + 1).read_to_end(&mut b).ok()?;
        total += b.len() as u64;
        if b.len() as u64 > MAX_ENTRY || total > MAX_TOTAL {
            return None;
        }
        files.push((name, b));
    }
    let mut changed = false;
    for (name, b) in &mut files {
        if !(name.ends_with(".xhtml") || name.ends_with(".html") || name.ends_with(".htm")) {
            continue;
        }
        let Ok(text) = std::str::from_utf8(b) else { continue };
        if !text.contains("noteref") && !text.contains("label") {
            continue;
        }
        if let Some(t) = take_notes(text, &mut notes) {
            *b = t.into_bytes();
            changed = true;
        }
    }
    if !changed {
        return None;
    }
    let mut z = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, b) in &files {
        z.start_file(name.as_str(), opts).ok()?;
        z.write_all(b).ok()?;
    }
    Some((z.finish().ok()?.into_inner(), notes))
}

/// Make each note mark in the document a note: the reference where the mark was, the note's
/// paragraphs in a footnote of the document's. Gives how many.
fn restore_notes(doc: &mut Document, notes: &std::collections::BTreeMap<String, Vec<String>>) -> usize {
    use jcj_doc::para::{InlineObject, NoteKind};
    use jcj_doc::props::CharProps;
    let mut made = 0;
    for i in 0..doc.body.len() {
        while let Some(p) = doc.body[i].as_para() {
            let Some(a) = p.text.find(REF_START) else { break };
            let Some(len) = p.text[a..].find(REF_END) else { break };
            let id = p.text[a + REF_START.len_utf8()..a + len].to_owned();
            let b = a + len + REF_END.len_utf8();
            let part = notes.get(&id).map(|paras| {
                let blocks = paras.iter().map(|t| jcj_doc::para_block(jcj_doc::Paragraph::with_text(t, CharProps::default()))).collect();
                doc.add_part(jcj_doc::PartKind::Footnote, blocks)
            });
            let DocBlock::Para(p) = Arc::make_mut(&mut doc.body[i]) else { break };
            if p.delete(a, b).is_err() {
                break;
            }
            if let Some(id) = part {
                let _ = p.insert_object(a, InlineObject::NoteRef { kind: NoteKind::Footnote, id, custom: String::new() }, &CharProps::default());
                made += 1;
            }
        }
    }
    made
}

/// Import an EPUB: its chapters through the core's reader, its structure from its package.
pub fn import(bytes: &[u8], rules: &ImportRules, title: &str) -> Result<Imported, Error> {
    // Notes: the core's reader reads text, so each reference becomes a mark and each note is
    // taken out first, then made a note again.
    let (doc, notes) = match without_notes(bytes) {
        Some((plain, notes)) => {
            let mut doc = crate::read(crate::Format::Epub, &plain)?;
            let n = restore_notes(&mut doc, &notes);
            (doc, n)
        }
        None => (crate::read(crate::Format::Epub, bytes)?, 0),
    };
    let Some((roles, ranges)) = roles(bytes).and_then(|r| chunks(&doc, r.len()).map(|c| (r, c))) else {
        return crate::import_document(&doc, rules, title);
    };
    let paras = crate::paras_of(&doc);
    let mut keep = vec![true; doc.body.len()];
    let mut forced: Vec<Option<Kind>> = vec![None; doc.body.len()];
    // In a document whose kind the package says, only its opening heading starts a section: any
    // other line detection takes for a heading is a subheading or a title page's line.
    let mut demote = vec![false; doc.body.len()];
    let mut dropped_nav = 0;
    let mut dropped_numbers = 0;
    let mut untitled = Vec::new();
    for (role, range) in roles.iter().zip(&ranges) {
        if role.nav {
            for k in range.clone() {
                keep[k] = false;
            }
            dropped_nav += 1;
            continue;
        }
        let Some(kind) = role.kind else { continue };
        for k in range.clone() {
            demote[k] = true;
        }
        // The document's first lines with words: a number line before a heading is Book's to make
        // again; the first heading (or the first line, for a part) starts the section.
        let lines: Vec<usize> = range.clone().filter(|&k| !detect::is_empty(&paras[k])).take(3).collect();
        let Some(&first) = lines.first() else { continue };
        let heading = |k: usize| paras[k].level.is_some() || paras[k].bold || paras[k].centered;
        let label = |k: usize| paras[k].text.starts_with(LABEL);
        match lines.get(1) {
            Some(&second) if (label(first) || number_line(&paras[first].text)) && heading(second) && !label(second) => {
                keep[first] = false;
                dropped_numbers += 1;
                forced[second] = Some(kind);
            }
            // A number alone ("Chapter One" and no title): an untitled section Book numbers.
            _ if label(first) && matches!(kind, Kind::Part | Kind::Chapter) => {
                untitled.push(first);
                dropped_numbers += 1;
                forced[first] = Some(kind);
            }
            _ if heading(first) || kind == Kind::Part => forced[first] = Some(kind),
            _ => {}
        }
    }
    let mut doc = doc;
    // The label marks go: they are not the book's words.
    for b in &mut doc.body {
        if b.as_para().is_some_and(|p| p.text.contains(LABEL))
            && let DocBlock::Para(p) = Arc::make_mut(b)
        {
            while let Some(a) = p.text.find(LABEL) {
                if p.delete(a, a + LABEL.len_utf8()).is_err() {
                    break;
                }
            }
        }
    }
    for k in untitled {
        if let Some(b) = doc.body.get_mut(k) {
            *b = Arc::new(DocBlock::Para(jcj_doc::Paragraph::new()));
        }
    }
    let mut kept = doc.clone();
    kept.body = doc.body.iter().zip(&keep).filter(|(_, k)| **k).map(|(b, _)| b.clone()).collect::<Vec<Arc<DocBlock>>>();
    let forced: Vec<(Option<Kind>, bool)> = forced.into_iter().zip(demote).zip(&keep).filter(|(_, k)| **k).map(|(f, _)| f).collect();
    let mut imported = crate::import_with(&kept, rules, title, &|marks: &mut Vec<Mark>| {
        for (m, (f, demote)) in marks.iter_mut().zip(&forced) {
            match f {
                Some(kind) => *m = Mark::Heading(*kind, "epub-structure"),
                None if *demote && matches!(m, Mark::Heading(..)) => *m = Mark::Body,
                None => {}
            }
        }
    })?;
    let r: &mut Report = &mut imported.report;
    if dropped_nav > 0 {
        r.dropped.insert("the ebook's own contents", dropped_nav);
    }
    if notes > 0 {
        r.kept.insert("notes", notes);
    }
    if dropped_numbers > 0 {
        r.dropped.insert("number lines (Book numbers parts and chapters)", dropped_numbers);
    }
    Ok(imported)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_lines_and_kinds() {
        for yes in ["Part One", "PART II.", "Chapter 12", "CHAPTER XII", "12", "XII", "Book the First"] {
            assert!(number_line(yes), "{yes}");
        }
        for no in ["A Voyage to Lilliput", "Chapter 12: The Storm Breaks Over Us", "Partly cloudy today and tomorrow"] {
            assert!(!number_line(no), "{no}");
        }
        assert_eq!(kind_of("bodymatter chapter"), (false, Some(Kind::Chapter)));
        assert_eq!(kind_of("frontmatter dedication"), (false, Some(Kind::Front)));
        assert_eq!(kind_of("part"), (false, Some(Kind::Part)));
        assert_eq!(kind_of("toc"), (true, None));
        assert_eq!(join("EPUB/package.opf", "text/a.xhtml#x"), "EPUB/text/a.xhtml");
        assert_eq!(join("OPS/content.opf", "../b.xhtml"), "b.xhtml");
    }
}
