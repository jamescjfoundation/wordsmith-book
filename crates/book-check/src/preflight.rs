// SPDX-License-Identifier: MIT OR Apache-2.0
//! Preflight (PRE-1, PRE-4, SPEC section 8): the built files read back and checked against the
//! edition's profile and the source, each finding with a severity, plain words and a link to its
//! page or section.
//!
//! Print: page size, margins for the page count, fonts embedded and licensed for embedding, the
//! smallest type, picture resolution, blank-page runs, the page count, annotations and bookmarks,
//! and every letter of the PDF against the book. Ebook: Book's structure checks, alternative
//! text, accessibility metadata against the content, the cover, and every word against the
//! source. Both: the metadata the book still needs and fixes waiting on the writer.

use std::collections::BTreeMap;

use wordsmith_book_design::fonts::{self, Embedding};
use wordsmith_book_format::source::Opened;
use wordsmith_book_model::{BookProject, Edition, Severity};
use wordsmith_book_typeset::Typeset;
use wordsmith_manuscript_format::book::Book;

use crate::build::{self, Print};
use crate::{epub, verify};

/// Where a finding points.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Link {
    /// Nowhere in particular.
    None,
    /// A page of the print edition, from 1.
    Page(usize),
    /// A piece of the book: a section's id or `element:<id>`.
    Section(String),
    /// An edition, by id.
    Edition(String),
    /// A file inside the EPUB.
    File(String),
}

/// One finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    /// The rule (`margins`, `fonts`, `type-size`, `pictures`, `blank-pages`, `page-count`,
    /// `annotations`, `page-size`, `words`, `epub`, `alt-text`, `accessibility`, `cover`,
    /// `metadata`, `fixes`, `validator`).
    pub rule: &'static str,
    /// How much it matters.
    pub severity: Severity,
    /// What, in plain words.
    pub message: String,
    /// Where.
    pub link: Link,
}

fn finding(rule: &'static str, severity: Severity, message: impl Into<String>, link: Link) -> Finding {
    Finding { rule, severity, message: message.into(), link }
}

/// A preflight report for one edition.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// The edition checked.
    pub edition: String,
    /// What was found, blocking findings first.
    pub findings: Vec<Finding>,
}

impl Report {
    /// Whether anything blocks export.
    pub fn blocked(&self) -> bool {
        self.findings.iter().any(|f| f.severity == Severity::Blocks)
    }

    /// The findings of one severity.
    pub fn of(&self, severity: Severity) -> Vec<&Finding> {
        self.findings.iter().filter(|f| f.severity == severity).collect()
    }

    fn sort(&mut self) {
        self.findings.sort_by_key(|f| match f.severity {
            Severity::Blocks => 0,
            Severity::Warning => 1,
            Severity::Note => 2,
        });
    }
}

const LIMITS: jcj_output::ReadLimits = jcj_output::ReadLimits { max_input_bytes: 512 << 20, max_pages: 10_000, max_glyphs_per_page: 50_000 };

/// The letters a PDF prints in each page's body area, in order (running heads and folios left
/// out), with the footnotes' letters apart.
pub fn pdf_text(t: &Typeset, pdf: &[u8]) -> Result<(String, String), String> {
    let pages = jcj_output::read_pages(pdf, &LIMITS).map_err(|e| e.to_string())?;
    if pages.len() != t.page_count() {
        return Err(format!("the PDF has {} pages; the layout has {}", pages.len(), t.page_count()));
    }
    let (mut body, mut notes) = (String::new(), String::new());
    for (i, p) in pages.iter().enumerate() {
        let (_, h) = t.page_size(i).unwrap_or((0.0, 0.0));
        let Some((_, y, _, bh)) = t.body_area(i) else { continue };
        let notes_top = t.notes_top(i).map(|n| f64::from(n) - 2.0);
        for g in &p.glyphs {
            let from_top = f64::from(h) - g.y;
            if from_top >= f64::from(y) - 3.0
                && from_top <= f64::from(y + bh) + 3.0
                && let Some(text) = &g.text
            {
                if notes_top.is_some_and(|n| from_top >= n) {
                    notes.push_str(text);
                } else {
                    body.push_str(text);
                }
            }
        }
    }
    Ok((body, notes))
}

/// The letters a PDF prints in each page's body area, footnotes left out.
pub fn pdf_body_text(t: &Typeset, pdf: &[u8]) -> Result<String, String> {
    pdf_text(t, pdf).map(|(b, _)| b)
}

fn letters(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

/// The margin findings: each side of each page against the profile's minimums, the inside
/// against the gutter the page count calls for.
pub fn margins(print: &Print) -> Vec<Finding> {
    let t = &print.typeset;
    let pages = t.page_count();
    let p = &print.profile;
    let gutter = p.gutter_for(pages);
    // Margins are measured from the trim; a page that bleeds needs more room outside.
    let b = t.bleed() * 72.0;
    let outside_min = if b > 0.0 { p.minimum.outside_bleed } else { p.minimum.outside };
    let mut out = Vec::new();
    for i in 0..pages {
        let (Some((w, h)), Some((x, y, bw, bh))) = (t.page_size(i), t.body_area(i)) else { continue };
        let inch = |v: f32| v / 72.0;
        // Right-hand pages (odd numbers) have the inside on the left; the bleed is outside.
        let recto = i % 2 == 0;
        let (bl, br) = if recto { (0.0, b) } else { (b, 0.0) };
        let (left, right, top, bottom) = (inch(x - bl), inch(w - x - bw - br), inch(y - b), inch(h - y - bh - b));
        let (inside, outside) = if recto { (left, right) } else { (right, left) };
        let mut problems = Vec::new();
        if inside + 0.001 < gutter {
            problems.push(format!("the inside margin is {inside:.3} in; a book of {pages} pages needs {gutter:.3} in"));
        }
        if outside + 0.001 < outside_min {
            problems.push(format!("the outside margin is {outside:.3} in; at least {outside_min:.3} in is needed"));
        }
        if top + 0.001 < p.minimum.top || bottom + 0.001 < p.minimum.bottom {
            problems.push(format!(
                "the top and bottom margins are {top:.3} and {bottom:.3} in; at least {:.3} and {:.3} in are needed",
                p.minimum.top, p.minimum.bottom
            ));
        }
        if !problems.is_empty() {
            out.push(finding("margins", Severity::Blocks, format!("Page {}: {}.", i + 1, problems.join("; ")), Link::Page(i + 1)));
            if out.len() >= 5 {
                out.push(finding("margins", Severity::Blocks, "Other pages have the same margin problem.", Link::None));
                break;
            }
        }
    }
    out
}

/// Fonts whose embedding bits forbid embedding, among `faces` (name, file bytes).
pub fn font_licenses(faces: &[(String, &[u8])]) -> Vec<Finding> {
    faces
        .iter()
        .filter(|(_, b)| fonts::embedding(b) == Embedding::Restricted)
        .map(|(n, _)| finding("fonts", Severity::Blocks, format!("The font {n} does not allow embedding; choose another."), Link::None))
        .collect()
}

/// The PDF's own facts: page size, fonts embedded, pictures' resolution, annotations, bookmarks.
pub fn pdf_facts(print: &Print, pdf: &[u8]) -> Vec<Finding> {
    let facts = match jcj_output::read_facts(pdf, &LIMITS) {
        Ok(f) => f,
        Err(e) => return vec![finding("page-size", Severity::Blocks, format!("The PDF could not be read back: {e}."), Link::None)],
    };
    let mut out = Vec::new();
    let (tw, th) = print.look.trim;
    let bleed = f64::from(print.look.bleed) * 72.0;
    for (i, p) in facts.pages.iter().enumerate() {
        // The trim box (or, without bleed, the page) is the trim size; with bleed the page is
        // larger by the bleed outside, top and bottom.
        let cut = if bleed > 0.0 { p.trim.unwrap_or(p.media) } else { p.media };
        let sheet_ok =
            (p.media.width() - (f64::from(tw) * 72.0 + bleed)).abs() <= 0.5 && (p.media.height() - (f64::from(th) * 72.0 + 2.0 * bleed)).abs() <= 0.5;
        if bleed > 0.0 && (p.trim.is_none() || p.bleed.is_none() || !sheet_ok) {
            out.push(finding(
                "page-size",
                Severity::Blocks,
                format!("Page {} does not carry its trim and bleed: it is set to bleed.", i + 1),
                Link::Page(i + 1),
            ));
            break;
        }
        if (cut.width() - f64::from(tw) * 72.0).abs() > 0.5 || (cut.height() - f64::from(th) * 72.0).abs() > 0.5 {
            out.push(finding(
                "page-size",
                Severity::Blocks,
                format!("Page {} is {:.2} × {:.2} in, not the trim size {tw} × {th} in.", i + 1, cut.width() / 72.0, cut.height() / 72.0),
                Link::Page(i + 1),
            ));
            break;
        }
    }
    for f in facts.unembedded_fonts() {
        out.push(finding("fonts", Severity::Blocks, format!("The font {} is not embedded in the PDF.", f.name), Link::None));
    }
    for (i, p) in facts.pages.iter().enumerate() {
        if !p.annotations.is_empty() {
            out.push(finding(
                "annotations",
                Severity::Blocks,
                format!("Page {} has {} (links or comments); print files must have none.", i + 1, p.annotations.join(", ")),
                Link::Page(i + 1),
            ));
        }
        for im in &p.images {
            if !im.mask && im.ppi() < 300.0 {
                out.push(finding(
                    "pictures",
                    Severity::Warning,
                    format!("A picture on page {} prints at {:.0} pixels per inch; 300 or more prints sharply.", i + 1, im.ppi()),
                    Link::Page(i + 1),
                ));
            }
        }
    }
    if facts.bookmarks > 0 {
        out.push(finding(
            "annotations",
            Severity::Blocks,
            format!("The PDF has {} bookmarks; print files must have none.", facts.bookmarks),
            Link::None,
        ));
    }
    out
}

/// Blank pages: no run longer than the profile allows inside the book or at its end.
pub fn blank_runs(blank: &[bool], inside: usize, end: usize) -> Vec<Finding> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < blank.len() {
        if !blank[i] {
            i += 1;
            continue;
        }
        let start = i;
        while i < blank.len() && blank[i] {
            i += 1;
        }
        let run = i - start;
        let at_end = i == blank.len();
        let limit = if at_end { end } else { inside };
        if run > limit {
            let where_ = if at_end { "at the end of the book" } else { "inside the book" };
            out.push(finding(
                "blank-pages",
                Severity::Blocks,
                format!("Pages {}–{} are {run} blank pages in a row {where_}; at most {limit} are allowed.", start + 1, i),
                Link::Page(start + 1),
            ));
        }
    }
    out
}

/// The page count against the profile.
pub fn page_count(print: &Print, paper: wordsmith_book_model::Paper) -> Vec<Finding> {
    let p = &print.profile;
    let n = print.typeset.page_count();
    let max = p.max_pages.of(paper);
    let mut out = Vec::new();
    if n < p.min_pages {
        out.push(finding(
            "page-count",
            Severity::Blocks,
            format!("The book has {n} pages; the {} profile needs at least {}.", p.name, p.min_pages),
            Link::None,
        ));
    }
    if n > max {
        out.push(finding("page-count", Severity::Blocks, format!("The book has {n} pages; at most {max} fit on this paper."), Link::None));
    }
    if p.even_pages && n % 2 == 1 {
        out.push(finding(
            "page-count",
            Severity::Note,
            format!("The book has {n} pages; the printer adds a blank page to make it even."),
            Link::None,
        ));
    }
    out
}

/// Every letter of the PDF's body against the laid-out book, and the author's words against the
/// source.
pub fn print_words(print: &Print, pdf: &[u8], source: &Book) -> Vec<Finding> {
    let mut out = Vec::new();
    let printed: String = print.typeset.printed_paragraphs().iter().map(|p| p.0.as_str()).collect::<Vec<_>>().join(" ");
    let notes: String = print.typeset.printed_notes().join(" ");
    match pdf_text(&print.typeset, pdf) {
        Err(e) => out.push(finding("words", Severity::Blocks, format!("The PDF's text could not be read back: {e}."), Link::None)),
        Ok((text, note_text)) => {
            for (what, want, got) in [("text", letters(&printed), letters(&text)), ("footnotes", letters(&notes), letters(&note_text))] {
                if want != got {
                    let at = want.chars().zip(got.chars()).position(|(a, b)| a != b).unwrap_or(want.len().min(got.len()));
                    let around: String = want.chars().skip(at.saturating_sub(20)).take(40).collect();
                    out.push(finding("words", Severity::Blocks, format!("The PDF's {what} differs from the book's near “{around}”."), Link::None));
                }
            }
        }
    }
    let ours = wordsmith_book_typeset::from_book::source_words(&print.book);
    let theirs = crate::verify::book_words(source, true);
    if let Err(d) = verify::compare(&theirs, &ours) {
        // Which section the difference falls in.
        let mut seen = 0;
        let mut link = Link::None;
        for s in &print.book.sections {
            let one =
                wordsmith_book_typeset::PrintBook { title: String::new(), author: String::new(), lang: String::new(), sections: vec![s.clone()] };
            seen += wordsmith_book_typeset::from_book::source_words(&one).len();
            if seen > d.at {
                link = Link::Section(s.id.clone());
                break;
            }
        }
        out.push(finding("words", Severity::Blocks, format!("The print edition's words differ from the source's: {d}"), link));
    }
    out
}

/// Typographic problems by page (PRE-3): widows, orphans and stacked hyphens are warnings; loose
/// lines and rivers are notes, for a person to look at.
pub fn typography(print: &Print) -> Vec<Finding> {
    use wordsmith_book_typeset::TypoKind;
    let limit = if print.look.strict_hyphens { 1 } else { 2 };
    print
        .typeset
        .typography(limit, 0.6)
        .into_iter()
        .map(|t| {
            let (severity, what) = match t.kind {
                TypoKind::Orphan => (Severity::Warning, "a paragraph's first line is alone at the foot".to_owned()),
                TypoKind::Widow => (Severity::Warning, "a paragraph's last line is alone at the head".to_owned()),
                TypoKind::StackedHyphens => (Severity::Warning, format!("more than {limit} hyphenated lines in a row")),
                TypoKind::LooseLine => (Severity::Note, "a justified line has very wide word spaces".to_owned()),
                TypoKind::River => (Severity::Note, "word spaces line up down the page (a river)".to_owned()),
            };
            let at = if t.text.is_empty() { String::new() } else { format!(" (“{}…”)", t.text) };
            finding("typography", severity, format!("Page {}: {what}{at}.", t.page), Link::Page(t.page))
        })
        .collect()
}

/// Every print finding for a built print edition and its PDF.
pub fn print_findings(print: &Print, pdf: &[u8], source: &Book, edition: &Edition) -> Vec<Finding> {
    let mut out = margins(print);
    let faces: Vec<(String, &[u8])> = fonts::bundled()
        .into_iter()
        .filter(|f| {
            fonts::family_name(f.bytes).is_some_and(|n| n == print.look.body_font || n == print.look.heading_font)
                || [&print.look.body_font, &print.look.heading_font].iter().any(|n| n.as_str() == f.role.key())
        })
        .map(|f| (format!("{} {}", f.role.title(), f.style), f.bytes))
        .collect();
    out.extend(font_licenses(&faces));
    out.extend(pdf_facts(print, pdf));
    if let Some((size, text)) = print.typeset.smallest_type(&print.look)
        && size + 0.01 < print.profile.min_type
    {
        out.push(finding(
            "type-size",
            Severity::Blocks,
            format!("Some text is set at {size:.1} pt (“{text}”); nothing may print under {:.1} pt.", print.profile.min_type),
            Link::None,
        ));
    }
    if let Ok(pages) = jcj_output::read_pages(pdf, &LIMITS) {
        let blank: Vec<bool> = pages.iter().map(|p| !p.glyphs.iter().any(|g| g.text.as_deref().is_some_and(|t| !t.trim().is_empty()))).collect();
        out.extend(blank_runs(&blank, print.profile.blank_run_inside, print.profile.blank_run_end));
    }
    out.extend(page_count(print, edition.paper.unwrap_or_default()));
    out.extend(typography(print));
    out.extend(print_words(print, pdf, source));
    out
}

/// The accessibility metadata of an EPUB against its content (SPEC 6.5).
pub fn accessibility(bytes: &[u8]) -> Vec<Finding> {
    let files = match epub::unzip(bytes) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    let get = |p: &str| files.iter().find(|f| f.0 == p).and_then(|f| String::from_utf8(f.2.clone()).ok());
    let Some(opf_path) = get("META-INF/container.xml").and_then(|c| {
        let d = epub::parse(&c).ok()?;
        d.descendants().find_map(|n| n.attribute("full-path")).map(str::to_owned)
    }) else {
        return Vec::new();
    };
    let Some(opf_text) = get(&opf_path) else { return Vec::new() };
    let Ok(opf) = epub::parse(&opf_text) else { return Vec::new() };
    let meta: Vec<(String, String)> =
        opf.descendants().filter_map(|n| Some((n.attribute("property")?.to_owned(), n.text().unwrap_or("").trim().to_owned()))).collect();
    let has = |k: &str, v: &str| meta.iter().any(|(a, b)| a == k && b == v);
    let any = |k: &str| meta.iter().any(|(a, _)| a == k);
    let link = Link::File(opf_path.clone());
    let mut out = Vec::new();
    for k in [
        "schema:accessMode",
        "schema:accessModeSufficient",
        "schema:accessibilityFeature",
        "schema:accessibilityHazard",
        "schema:accessibilitySummary",
        "dcterms:conformsTo",
    ] {
        if !any(k) {
            out.push(finding("accessibility", Severity::Blocks, format!("The ebook does not say its {k}."), link.clone()));
        }
    }
    // Pictures that carry meaning: shown with alternative text, outside the cover page.
    let meaningful = files.iter().filter(|(n, _, _)| n.ends_with(".xhtml")).any(|(_, _, b)| {
        let Ok(t) = std::str::from_utf8(b) else { return false };
        let Ok(d) = epub::parse(t) else { return false };
        let cover = d.descendants().any(|n| n.attribute(("http://www.idpf.org/2007/ops", "type")) == Some("cover"));
        !cover && d.descendants().any(|n| n.has_tag_name("img") && n.attribute("alt").is_some_and(|a| !a.trim().is_empty()))
    });
    if meaningful && !has("schema:accessMode", "visual") {
        out.push(finding("accessibility", Severity::Blocks, "The ebook has pictures but does not list visual access.", link.clone()));
    }
    if meaningful && !has("schema:accessibilityFeature", "alternativeText") {
        out.push(finding("accessibility", Severity::Blocks, "The ebook has pictures but does not list their alternative text.", link.clone()));
    }
    if !meaningful && has("schema:accessMode", "visual") {
        out.push(finding("accessibility", Severity::Warning, "The ebook lists visual access but shows no pictures.", link));
    }
    out
}

/// Every ebook finding for a built EPUB.
pub fn ebook_findings(bytes: &[u8], source: &Book, cover_problems: &[String], has_cover: bool, notes_last: bool) -> Vec<Finding> {
    let mut out: Vec<Finding> = epub::check(bytes)
        .into_iter()
        .take(50)
        .map(|p| {
            let link = if p.file.is_empty() { Link::None } else { Link::File(p.file.clone()) };
            finding("epub", Severity::Blocks, p.to_string(), link)
        })
        .collect();
    out.extend(accessibility(bytes));
    match epub::words(bytes, true) {
        Ok(words) => {
            let want = if notes_last { verify::book_words_notes_last(source) } else { verify::book_words(source, true) };
            if let Err(d) = verify::compare(&want, &words) {
                out.push(finding("words", Severity::Blocks, format!("The ebook's words differ from the source's: {d}"), Link::None));
            }
        }
        Err(e) => out.push(finding("words", Severity::Blocks, format!("The ebook's text could not be read back: {e}."), Link::None)),
    }
    for c in cover_problems {
        out.push(finding("cover", Severity::Warning, format!("{}.", capital(c)), Link::None));
    }
    if !has_cover {
        out.push(finding("cover", Severity::Warning, "The ebook has no cover; ebook stores ask for one.", Link::None));
    }
    out
}

fn capital(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

/// What the book still needs (metadata, ISBNs) and fixes waiting on the writer.
pub fn book_findings(project: &BookProject, source: &Opened) -> Vec<Finding> {
    let mut out: Vec<Finding> = wordsmith_book_format::needs::needs(project, &source.book)
        .into_iter()
        .map(|n| finding("metadata", n.severity, n.message, n.edition.map_or(Link::None, Link::Edition)))
        .collect();
    let states = wordsmith_book_format::fixes::states(&project.fixes, source);
    let waiting = states.iter().filter(|s| **s == wordsmith_book_format::fixes::State::Waiting).count();
    if waiting > 0 {
        let s = if waiting == 1 { "fix" } else { "fixes" };
        out.push(finding("fixes", Severity::Note, format!("{waiting} {s} not yet accepted by the writer (they are in this build)."), Link::None));
    }
    out
}

/// Pictures without alternative text, as findings linked to the sections that show them.
fn alt_text_findings(a: &wordsmith_book_format::assemble::Assembled, decorative: &[String]) -> Vec<Finding> {
    let missing = wordsmith_book_epub::access::missing_alt_text(a, decorative);
    missing
        .into_iter()
        .map(|src| {
            let piece =
                a.pieces().find(|p| p.blocks.iter().any(|b| matches!(b, wordsmith_manuscript_model::Block::Image { src: s, .. } if *s == src)));
            finding(
                "alt-text",
                Severity::Blocks,
                format!("The picture {src} needs alternative text, or to be marked decorative."),
                piece.map_or(Link::None, |p| Link::Section(p.id.clone())),
            )
        })
        .collect()
}

/// Build an edition and preflight it. `images` gives a picture's bytes by its path. Gives the
/// report and the built bytes (the PDF or the EPUB), when the build got that far.
pub fn preflight(
    project: &BookProject,
    designs: &BTreeMap<String, String>,
    source: &Opened,
    edition: &Edition,
    images: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> (Report, Option<Vec<u8>>) {
    let book = wordsmith_book_format::fixes::book_for_build(&project.fixes, source);
    // The words each output must hold: the book as that output shows it (STR-9).
    let shown = wordsmith_book_format::assemble::shown_in(project, &book, edition.format.is_print());
    let mut report = Report { edition: edition.id.clone(), findings: book_findings(project, source) };
    let mut bytes = None;
    if edition.format.is_print() {
        match build::print(project, designs, &book, edition, images) {
            Ok(print) => match print.typeset.pdf() {
                Ok(pdf) => {
                    report.findings.extend(print_findings(&print, &pdf, &shown, edition));
                    bytes = Some(pdf);
                }
                Err(e) => report.findings.push(finding("page-size", Severity::Blocks, format!("The PDF could not be written: {e}."), Link::None)),
            },
            Err(e) => report.findings.push(finding("page-size", Severity::Blocks, format!("The book could not be laid out: {e}."), Link::None)),
        }
    } else {
        let a = wordsmith_book_format::assemble::assemble(project, &book, Some(edition));
        let alt = alt_text_findings(&a, &project.decorative);
        if alt.is_empty() {
            match build::ebook(project, designs, &book, edition, images) {
                Ok((_, e)) => {
                    report.findings.extend(ebook_findings(
                        &e.bytes,
                        &shown,
                        &e.cover_problems,
                        !edition.cover.is_empty(),
                        edition.notes == wordsmith_book_model::NoteStyle::BookEnd,
                    ));
                    for f in &e.fonts_refused {
                        report.findings.push(finding(
                            "fonts",
                            Severity::Warning,
                            format!("The font {f} does not allow embedding; the ebook leaves it out."),
                            Link::None,
                        ));
                    }
                    bytes = Some(e.bytes);
                }
                Err(e) => report.findings.push(finding("epub", Severity::Blocks, format!("The ebook could not be made: {e}."), Link::None)),
            }
        } else {
            report.findings.extend(alt);
        }
    }
    report.sort();
    (report, bytes)
}

/// Add the external EPUB validator's findings to an ebook's report (a note when it is not set
/// up).
pub fn add_validator(report: &mut Report, epub: &[u8], settings: &crate::validator::Settings) {
    report.findings.extend(crate::validator::check(settings, epub));
    report.sort();
}
