// SPDX-License-Identifier: MIT OR Apache-2.0
//! The ebook: XHTML, CSS from the design, the package document, navigation and accessibility metadata.
//!
//! [`ebook`] turns an assembled book (`book-format`'s [`Assembled`]) and its design into an EPUB
//! 3 (EBK-1, EBK-2): one XHTML file per piece, one stylesheet made from the design's ebook
//! choices ([`css`]), a navigation document with every part and chapter and the main front and
//! back matter, landmarks, an NCX, and the package around them ([`package`], C6).

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod css;
pub mod package;

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use wordsmith_book_design::design::{BreakStyle, Design, FirstLine};
use wordsmith_book_format::assemble::{Assembled, Piece, What};
use wordsmith_book_model::{Edition, ElementKind, Only};
use wordsmith_manuscript_format::book::FrontKind;
use wordsmith_manuscript_model::{Block, Fence, ImageRole, Inline, Inlines, Note};

use package::{Item, Landmark, Metadata, NavPoint, Package, escape};

/// What a book's ebook needs beyond the assembled book and its design.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    /// The book's language (BCP 47); English when empty.
    pub language: String,
    /// When the project was last changed (`YYYY-MM-DD`), for `dcterms:modified`, so the same
    /// project gives the same bytes.
    pub date: String,
    /// A seed for the identifier when the edition has no ISBN (the project's creation date and
    /// the book's title and author are used with it).
    pub seed: String,
}

/// What the ebook left out, for the build report.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Left {
    /// Raw blocks for other formats.
    pub raw: usize,
    /// Pictures whose file was missing or not a PNG, JPEG, GIF or SVG.
    pub pictures: Vec<String>,
}

/// A built ebook.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ebook {
    /// The EPUB's bytes.
    pub bytes: Vec<u8>,
    /// The package it was written from.
    pub package: Package,
    /// What was left out.
    pub left: Left,
}

/// Why an ebook could not be made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The edition is not an ebook.
    NotEbook,
    /// The package could not be written.
    Package(package::PackageError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotEbook => write!(f, "the edition is not an ebook"),
            Error::Package(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

/// Whether a piece is in the ebook: not print-only, and not a half title (a printed book's
/// convention that an ebook does without).
pub fn in_ebook(p: &Piece) -> bool {
    p.only != Only::Print && p.what != What::Element(ElementKind::HalfTitle)
}

/// The `epub:type` and ARIA role of a piece (EPUB 3 Structural Semantics, DPUB-ARIA).
pub fn semantics(p: &Piece) -> (&'static str, Option<&'static str>) {
    match p.what {
        What::Part => ("part", Some("doc-part")),
        What::Chapter | What::Other => ("chapter", Some("doc-chapter")),
        What::Element(k) => match k {
            ElementKind::HalfTitle => ("halftitlepage", None),
            ElementKind::TitlePage => ("titlepage", None),
            ElementKind::Copyright => ("copyright-page", None),
            ElementKind::Dedication => ("dedication", Some("doc-dedication")),
            ElementKind::Epigraph => ("epigraph", Some("doc-epigraph")),
            ElementKind::Contents => ("toc", Some("doc-toc")),
            ElementKind::Acknowledgements => ("acknowledgments", Some("doc-acknowledgments")),
            ElementKind::Foreword => ("foreword", Some("doc-foreword")),
            ElementKind::Preface => ("preface", Some("doc-preface")),
            ElementKind::Introduction => ("introduction", Some("doc-introduction")),
            ElementKind::Prologue => ("prologue", Some("doc-prologue")),
            ElementKind::Epilogue => ("epilogue", Some("doc-epilogue")),
            ElementKind::Afterword => ("afterword", Some("doc-afterword")),
            ElementKind::Glossary => ("glossary", Some("doc-glossary")),
            ElementKind::Endnotes => ("endnotes", Some("doc-endnotes")),
            ElementKind::AboutTheAuthor => ("contributors", None),
            ElementKind::AlsoBy => ("other-credits", None),
            _ => ("", None),
        },
        What::Matter(k) => match k {
            FrontKind::TitlePage => ("titlepage", None),
            FrontKind::Copyright => ("copyright-page", None),
            FrontKind::Dedication => ("dedication", Some("doc-dedication")),
            FrontKind::Epigraph => ("epigraph", Some("doc-epigraph")),
            FrontKind::Contents => ("toc", Some("doc-toc")),
            FrontKind::Acknowledgements => ("acknowledgments", Some("doc-acknowledgments")),
            FrontKind::AboutTheAuthor => ("contributors", None),
            FrontKind::AlsoBy => ("other-credits", None),
            FrontKind::Other => ("", None),
        },
    }
}

/// Whether a piece is a short centered page (title page, copyright, dedication, epigraph).
fn plain(p: &Piece) -> bool {
    matches!(
        p.what,
        What::Element(ElementKind::HalfTitle | ElementKind::TitlePage | ElementKind::Copyright | ElementKind::Dedication | ElementKind::Epigraph)
            | What::Matter(FrontKind::TitlePage | FrontKind::Copyright | FrontKind::Dedication | FrontKind::Epigraph)
    )
}

/// Whether the source's contents page or Book's contents element stands for the navigation
/// document.
fn is_contents(p: &Piece) -> bool {
    matches!(p.what, What::Element(ElementKind::Contents) | What::Matter(FrontKind::Contents))
}

fn label_and_title(p: &Piece) -> String {
    let parts: Vec<&str> = [p.label.as_deref().unwrap_or(""), p.title.as_str()].into_iter().filter(|s| !s.is_empty()).collect();
    parts.join(": ")
}

/// A picture's media type from its first bytes.
pub fn media_type(b: &[u8]) -> Option<&'static str> {
    if b.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png")
    } else if b.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if b.starts_with(b"GIF87a") || b.starts_with(b"GIF89a") {
        Some("image/gif")
    } else if b.len() < 8 << 20 && std::str::from_utf8(b).is_ok_and(|s| s.contains("<svg")) {
        Some("image/svg+xml")
    } else {
        None
    }
}

fn extension(media: &str) -> &'static str {
    match media {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        _ => "svg",
    }
}

/// A name-based UUID (version 8, from SHA-256) of `text`, as `urn:uuid:…`.
pub fn uuid_urn(text: &str) -> String {
    let h = Sha256::digest(text.as_bytes());
    let mut b = [0u8; 16];
    b.copy_from_slice(&h[..16]);
    b[6] = (b[6] & 0x0F) | 0x80;
    b[8] = (b[8] & 0x3F) | 0x80;
    let hex: String = b.iter().map(|x| format!("{x:02x}")).collect();
    format!("urn:uuid:{}-{}-{}-{}-{}", &hex[0..8], &hex[8..12], &hex[12..16], &hex[16..20], &hex[20..32])
}

/// The identifier: the ebook ISBN as `urn:isbn:…`, else a UUID made from the book's title,
/// author and `seed`, so the same book keeps it from build to build.
pub fn identifier(isbn: &str, title: &str, author: &str, seed: &str) -> String {
    let digits: String = isbn.chars().filter(|c| c.is_ascii_digit() || *c == 'X').collect();
    if digits.len() == 13 || digits.len() == 10 { format!("urn:isbn:{digits}") } else { uuid_urn(&format!("{title}\n{author}\n{seed}")) }
}

struct Writer<'a> {
    lang: &'a str,
    images: &'a dyn Fn(&str) -> Option<Vec<u8>>,
    /// Pictures by source path: (href from the package document, media type, bytes).
    pictures: BTreeMap<String, (String, String, Vec<u8>)>,
    left: Left,
    design: &'a Design,
}

impl Writer<'_> {
    fn inlines(&self, c: &Inlines, notes: &NoteRefs) -> String {
        let mut s = String::new();
        for i in c {
            match i {
                Inline::Span { text, marks } => {
                    let mut t = escape(text).replace('\n', "<br/>");
                    if marks.small_caps {
                        t = format!("<span class=\"sc\">{t}</span>");
                    }
                    if marks.strike {
                        t = format!("<span class=\"strike\">{t}</span>");
                    }
                    if marks.bold {
                        t = format!("<b>{t}</b>");
                    }
                    if marks.italic {
                        t = format!("<i>{t}</i>");
                    }
                    if let Some(l) = marks.lang.as_ref().filter(|l| l.as_str() != self.lang) {
                        let l = escape(l);
                        t = format!("<span lang=\"{l}\" xml:lang=\"{l}\">{t}</span>");
                    }
                    s += &t;
                }
                Inline::NoteRef { label, .. } => {
                    if let Some(n) = notes.number(label) {
                        let id = &notes.prefix;
                        s += &format!("<a class=\"noteref\" epub:type=\"noteref\" role=\"doc-noteref\" id=\"{id}r{n}\" href=\"#{id}n{n}\">{n}</a>");
                    }
                }
            }
        }
        s
    }

    fn picture(&mut self, src: &str) -> Option<String> {
        if let Some((href, _, _)) = self.pictures.get(src) {
            return Some(href.clone());
        }
        let bytes = (self.images)(src)?;
        let media = media_type(&bytes)?;
        let href = format!("images/i{:03}.{}", self.pictures.len() + 1, extension(media));
        self.pictures.insert(src.to_owned(), (href.clone(), media.to_owned(), bytes));
        Some(href)
    }

    fn blocks(&mut self, blocks: &[Block], notes: &NoteRefs, opening: bool, out: &mut String) {
        let mut first = opening;
        for b in blocks {
            match b {
                Block::Para(c) => {
                    let inner = self.inlines(c, notes);
                    if first && !inner.is_empty() {
                        out.push_str(&format!("<p class=\"first\">{}</p>\n", self.opening(&inner)));
                        first = false;
                    } else {
                        out.push_str(&format!("<p>{inner}</p>\n"));
                    }
                }
                Block::Heading { level, content } => {
                    let h = (*level).clamp(1, 3) + 1;
                    out.push_str(&format!("<h{h}>{}</h{h}>\n", self.inlines(content, notes)));
                }
                Block::SceneBreak => {
                    let mark = &self.design.scene_break.mark;
                    if self.design.scene_break.style == BreakStyle::Mark && !mark.is_empty() {
                        out.push_str(&format!("<p class=\"break\" role=\"separator\">{}</p>\n", escape(mark)));
                    } else {
                        out.push_str("<hr class=\"break\"/>\n");
                    }
                }
                Block::Quote(ps) => {
                    out.push_str("<blockquote>\n");
                    for p in ps {
                        out.push_str(&format!("<p>{}</p>\n", self.inlines(p, notes)));
                    }
                    out.push_str("</blockquote>\n");
                }
                Block::Fenced { kind: Fence::Aside(_), paras } => {
                    out.push_str("<aside>\n");
                    for p in paras {
                        out.push_str(&format!("<p>{}</p>\n", self.inlines(p, notes)));
                    }
                    out.push_str("</aside>\n");
                }
                Block::Fenced { kind: Fence::Verse, paras } => {
                    out.push_str("<div class=\"verse\">\n");
                    for stanza in paras {
                        out.push_str(&format!("<p>{}</p>\n", self.inlines(stanza, notes)));
                    }
                    out.push_str("</div>\n");
                }
                Block::Image { alt, src, role } => match self.picture(src) {
                    Some(href) => {
                        let class = match role {
                            ImageRole::Inline => "inline",
                            ImageRole::Wide => "wide",
                            ImageRole::Page => "page",
                        };
                        out.push_str(&format!("<figure class=\"{class}\"><img src=\"../{}\" alt=\"{}\"/></figure>\n", escape(&href), escape(alt)));
                    }
                    None => self.left.pictures.push(src.clone()),
                },
                Block::Raw { .. } => self.left.raw += 1,
            }
        }
    }

    /// A piece's `<section>`, and whether it has a heading.
    fn section(&mut self, p: &Piece, id: &str, kind: &str, role: Option<&str>, refs: &NoteRefs) -> (String, bool) {
        let mut body = String::new();
        let types = kind.to_owned();
        let type_attr = if types.is_empty() { String::new() } else { format!(" epub:type=\"{types}\"") };
        let role_attr = role.map(|r| format!(" role=\"{r}\"")).unwrap_or_default();
        let class = if plain(p) {
            " class=\"matter-plain\""
        } else if p.what == What::Part {
            " class=\"part\""
        } else {
            ""
        };
        let heading = !plain(p) && (!p.title.is_empty() || p.label.is_some());
        let labelled = if heading { format!(" aria-labelledby=\"{id}-h\"") } else { String::new() };
        body += &format!("<section id=\"{id}\"{type_attr}{role_attr}{class}{labelled}>\n");
        if heading {
            body += "<header>\n";
            match (&p.label, p.title.is_empty()) {
                (Some(l), false) => body += &format!("<p class=\"label\">{}</p>\n<h1 id=\"{id}-h\">{}</h1>\n", escape(l), escape(&p.title)),
                (Some(l), true) => body += &format!("<h1 id=\"{id}-h\">{}</h1>\n", escape(l)),
                (None, _) => body += &format!("<h1 id=\"{id}-h\">{}</h1>\n", escape(&p.title)),
            }
            if !p.subtitle.is_empty() {
                body += &format!("<p class=\"subtitle\">{}</p>\n", escape(&p.subtitle));
            }
            body += "</header>\n";
        }
        if let Some((text, by)) = &p.epigraph {
            body += &format!("<blockquote class=\"epigraph\"><p>{}</p>", escape(text));
            if !by.is_empty() {
                body += &format!("<p class=\"attribution\">{}</p>", escape(by));
            }
            body += "</blockquote>\n";
        }
        let opening = matches!(p.what, What::Chapter | What::Other);
        self.blocks(&p.blocks, refs, opening, &mut body);
        self.notes(&p.notes, refs, &mut body);
        body += "</section>\n";
        (body, heading)
    }

    /// The first paragraph of a chapter: a drop cap or small-capital lead-in, as the design says.
    fn opening(&self, inner: &str) -> String {
        match self.design.ebook.first_line {
            FirstLine::None => inner.to_owned(),
            FirstLine::SmallCaps | FirstLine::DropCap => {
                // The lead-in is the first words up to the first markup or the fourth space.
                let end = inner.find('<').unwrap_or(inner.len());
                let plain = &inner[..end];
                let cut = plain.char_indices().filter(|(_, c)| *c == ' ').nth(3).map_or(plain.len(), |(i, _)| i);
                let (lead, rest) = inner.split_at(cut);
                if lead.trim().is_empty() {
                    return inner.to_owned();
                }
                if self.design.ebook.first_line == FirstLine::DropCap {
                    let mut chars = lead.char_indices();
                    let first_end = chars.find(|(_, c)| c.is_alphanumeric()).map(|(i, c)| i + c.len_utf8());
                    // A drop cap only on an ordinary letter (not inside an entity).
                    if let Some(e) = first_end.filter(|_| !lead.starts_with('&')) {
                        return format!("<span class=\"drop\">{}</span><span class=\"lead\">{}</span>{rest}", &lead[..e], &lead[e..]);
                    }
                }
                format!("<span class=\"lead\">{lead}</span>{rest}")
            }
        }
    }

    fn notes(&self, notes: &[Note], refs: &NoteRefs, out: &mut String) {
        if notes.is_empty() {
            return;
        }
        let id = &refs.prefix;
        out.push_str("<section class=\"notes\" epub:type=\"endnotes\" role=\"doc-endnotes\">\n<h2>Notes</h2>\n<ol>\n");
        for (i, n) in notes.iter().enumerate() {
            let k = i + 1;
            out.push_str(&format!("<li id=\"{id}n{k}\" epub:type=\"endnote\" role=\"doc-endnote\">"));
            for (j, p) in n.paras.iter().enumerate() {
                let back = if j + 1 == n.paras.len() {
                    format!(" <a href=\"#{id}r{k}\" role=\"doc-backlink\" title=\"Back to the text\">↩</a>")
                } else {
                    String::new()
                };
                out.push_str(&format!("<p>{}{back}</p>", self.inlines(p, &NoteRefs::default())));
            }
            out.push_str("</li>\n");
        }
        out.push_str("</ol>\n</section>\n");
    }
}

/// A piece's notes by label, numbered from 1, with an id prefix for the piece.
#[derive(Default)]
struct NoteRefs {
    prefix: String,
    labels: Vec<String>,
}

impl NoteRefs {
    fn number(&self, label: &str) -> Option<usize> {
        self.labels.iter().position(|l| l == label).map(|p| p + 1)
    }
}

fn document(lang: &str, title: &str, body_type: &str, body: &str) -> String {
    let lang = escape(lang);
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE html>\n<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\" lang=\"{lang}\" xml:lang=\"{lang}\">\n<head>\n<meta charset=\"UTF-8\"/>\n<title>{}</title>\n<link rel=\"stylesheet\" type=\"text/css\" href=\"../css/book.css\"/>\n</head>\n<body epub:type=\"{body_type}\">\n{body}</body>\n</html>\n",
        escape(title)
    )
}

/// The ebook of an assembled book in a design. `images` gives a picture's bytes by its path in
/// the source; pictures that are missing or of another kind are left out and reported.
pub fn ebook(
    a: &Assembled,
    design: &Design,
    edition: &Edition,
    settings: &Settings,
    images: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Result<Ebook, Error> {
    if edition.format.is_print() {
        return Err(Error::NotEbook);
    }
    let lang = if settings.language.is_empty() { "en" } else { settings.language.as_str() };
    let mut w = Writer { lang, images, pictures: BTreeMap::new(), left: Left::default(), design };
    let mut items = Vec::new();
    let mut spine = Vec::new();
    let mut nav_in_spine = None;
    let mut toc: Vec<NavPoint> = Vec::new();
    let mut landmarks = Vec::new();
    let mut first_body = true;
    let mut in_part;
    let mut n = 0;
    for (zone, pieces) in [("frontmatter", &a.front), ("bodymatter", &a.body), ("backmatter", &a.back)] {
        in_part = false;
        for p in pieces.iter().filter(|p| in_ebook(p)) {
            if is_contents(p) {
                // The navigation document is the contents, read where the contents stand.
                nav_in_spine.get_or_insert(spine.len());
                landmarks.push(Landmark { kind: "toc".into(), label: "Contents".into(), href: package::NAV.1.into() });
                continue;
            }
            n += 1;
            let id = format!("s{n:03}");
            let href = format!("text/{id}.xhtml");
            let (kind, role) = semantics(p);
            let refs = NoteRefs { prefix: format!("{id}-"), labels: p.notes.iter().map(|x| x.label.clone()).collect() };
            let (body, heading) = w.section(p, &id, kind, role, &refs);
            let title = if heading { label_and_title(p) } else { a.meta.title.clone() };
            items.push(Item {
                id: id.clone(),
                href: href.clone(),
                media_type: "application/xhtml+xml".into(),
                properties: Vec::new(),
                bytes: document(lang, &title, zone, &body).into_bytes(),
            });
            spine.push((id.clone(), true));
            // Navigation: parts hold their chapters; the main front and back matter are listed.
            let target = format!("{href}#{id}");
            let entry = NavPoint { label: if heading { label_and_title(p) } else { title_of(p) }, href: target.clone(), children: Vec::new() };
            match p.what {
                What::Part => {
                    in_part = true;
                    toc.push(entry);
                }
                What::Chapter | What::Other => match toc.last_mut() {
                    Some(part) if in_part => part.children.push(entry),
                    _ => toc.push(entry),
                },
                _ if zone != "frontmatter"
                    || !plain(p)
                    || matches!(p.what, What::Element(ElementKind::TitlePage) | What::Matter(FrontKind::TitlePage)) =>
                {
                    toc.push(entry)
                }
                _ => {}
            }
            if zone == "bodymatter" && first_body {
                first_body = false;
                landmarks.push(Landmark { kind: "bodymatter".into(), label: "Start of the book".into(), href: target.clone() });
            }
            match kind {
                "titlepage" => landmarks.push(Landmark { kind: "titlepage".into(), label: "Title page".into(), href: target.clone() }),
                "copyright-page" => landmarks.push(Landmark { kind: "copyright-page".into(), label: "Copyright".into(), href: target.clone() }),
                _ => {}
            }
            if zone == "backmatter" && !landmarks.iter().any(|l| l.kind == "backmatter") {
                landmarks.push(Landmark { kind: "backmatter".into(), label: "Back matter".into(), href: target });
            }
        }
    }
    let mut manifest = vec![Item {
        id: "css".into(),
        href: "css/book.css".into(),
        media_type: "text/css".into(),
        properties: Vec::new(),
        bytes: css::stylesheet(design, &css::Fonts::default()).into_bytes(),
    }];
    manifest.extend(items);
    for (i, (_, (href, media, bytes))) in w.pictures.iter().enumerate() {
        manifest.push(Item {
            id: format!("img{:03}", i + 1),
            href: href.clone(),
            media_type: media.clone(),
            properties: Vec::new(),
            bytes: bytes.clone(),
        });
    }
    let meta = &a.meta;
    let mut dc = Vec::new();
    if !meta.imprint.is_empty() {
        dc.push(("publisher".to_owned(), meta.imprint.clone()));
    }
    if !meta.rights.is_empty() {
        dc.push(("rights".to_owned(), meta.rights.clone()));
    }
    if !meta.year.is_empty() {
        dc.push(("date".to_owned(), meta.year.clone()));
    }
    let date = if settings.date.len() >= 10 { &settings.date[..10] } else { "2000-01-01" };
    let package = Package {
        metadata: Metadata {
            identifier: identifier(&edition.isbn, &meta.title, &meta.byline, &settings.seed),
            title: meta.title.clone(),
            language: lang.to_owned(),
            creators: if meta.byline.is_empty() { Vec::new() } else { vec![meta.byline.clone()] },
            modified: format!("{date}T00:00:00Z"),
            dc,
            meta: Vec::new(),
            links: Vec::new(),
        },
        items: manifest,
        spine,
        nav_in_spine,
        nav_title: "Contents".into(),
        nav_css: Some("css/book.css".into()),
        toc,
        landmarks,
    };
    let bytes = package::write(&package).map_err(Error::Package)?;
    Ok(Ebook { bytes, package, left: w.left })
}

/// A plain page's name in the contents.
fn title_of(p: &Piece) -> String {
    match p.what {
        What::Element(ElementKind::TitlePage) | What::Matter(FrontKind::TitlePage) => "Title Page".into(),
        What::Element(ElementKind::Copyright) | What::Matter(FrontKind::Copyright) => "Copyright".into(),
        What::Element(ElementKind::Dedication) | What::Matter(FrontKind::Dedication) => "Dedication".into(),
        What::Element(ElementKind::Epigraph) | What::Matter(FrontKind::Epigraph) => "Epigraph".into(),
        _ => p.title.clone(),
    }
}
