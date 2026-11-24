// SPDX-License-Identifier: MIT OR Apache-2.0
//! Print: a book and a design to pages, PDF, page images and the page map (plan section 5.6).
//!
//! Decision 0003: the embedded engine the plan proposed does not build against the core's
//! patched PDF libraries, so print takes the plan's fallback. A [`PrintBook`] (the book's
//! sections and blocks, already in reading order) and a [`Look`] (the design's values for one
//! edition) become a print document in the core's model (`jcj-doc`): mirrored margins, a section
//! per chapter opening on a right-hand page when the design says so, running heads on facing
//! pages and folios, small caps, drop caps, hyphenation and widow and orphan control. The core
//! lays it out (`jcj-layout`) and writes the PDF and page images (`jcj-output`). Book's text is
//! always data in the document's runs, never instructions.
//!
// core-candidate: open-source-core#19 (C5, a typesetting layer for books)

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

mod build;
pub mod fonts;

use jcj_doc::Document;
use jcj_layout::{DocLayout, LayoutCache};

pub use build::print_document;

/// Points per inch.
pub const INCH: f32 = 72.0;

/// The name PDFs give as the program that made them.
pub const CREATOR: &str = "Wordsmith Book";

/// Why a build failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The book or the look cannot be set (a missing font, a page too small for its margins).
    Setup(String),
    /// The PDF could not be written.
    Pdf(String),
    /// A page does not exist or could not be drawn.
    Page(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Setup(m) => write!(f, "the print build could not start: {m}"),
            Error::Pdf(m) => write!(f, "the PDF could not be written: {m}"),
            Error::Page(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for Error {}

/// A run of text with its marks. The text is the author's, exactly.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Span {
    /// The words.
    pub text: String,
    /// Italic.
    pub italic: bool,
    /// Bold.
    pub bold: bool,
    /// Small capitals.
    pub small_caps: bool,
}

impl Span {
    /// Plain text.
    pub fn plain(text: impl Into<String>) -> Span {
        Span { text: text.into(), ..Span::default() }
    }
}

/// A block of a section's text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    /// A body paragraph.
    Para(Vec<Span>),
    /// A scene break.
    Break,
    /// Verse: lines of one stanza.
    Verse(Vec<Vec<Span>>),
    /// A block quote of paragraphs.
    Quote(Vec<Vec<Span>>),
    /// A subheading.
    Heading(Vec<Span>),
}

/// One section that opens on a new page: a chapter, a part page, a front or back matter page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    /// Its stable id (for the page map).
    pub id: String,
    /// The number line ("Chapter One"), if shown.
    pub number: Option<String>,
    /// The title, if shown.
    pub title: String,
    /// The text.
    pub blocks: Vec<Block>,
}

/// The book to print, in reading order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PrintBook {
    /// The title (running heads, the PDF's title).
    pub title: String,
    /// The byline (running heads, the PDF's author).
    pub author: String,
    /// The language (BCP 47).
    pub lang: String,
    /// The sections.
    pub sections: Vec<Section>,
}

/// The first line of a chapter.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FirstLine {
    /// Nothing special.
    #[default]
    Plain,
    /// A drop cap of this many lines.
    DropCap(u8),
    /// The first words in small capitals.
    SmallCaps,
}

/// The design's values for one edition, in points unless named otherwise.
#[derive(Clone, Debug, PartialEq)]
pub struct Look {
    /// Trim width and height, inches.
    pub trim: (f32, f32),
    /// Inside (gutter side), outside, top and bottom margins, inches.
    pub margins: (f32, f32, f32, f32),
    /// The body font family.
    pub body_font: String,
    /// The heading font family.
    pub heading_font: String,
    /// Body size.
    pub body_size: f32,
    /// Line height as a multiple of the body size.
    pub leading: f32,
    /// First-line indent, ems.
    pub indent: f32,
    /// Hyphenate.
    pub hyphenate: bool,
    /// Chapters start on a right-hand page.
    pub recto: bool,
    /// The chapter opening's sink from the top margin, inches.
    pub sink: f32,
    /// The number line's size.
    pub number_size: f32,
    /// The title's size.
    pub title_size: f32,
    /// An ornament under the title ("" for none).
    pub ornament: String,
    /// The first line of each chapter.
    pub first_line: FirstLine,
    /// The scene break's mark ("" for a blank line).
    pub break_mark: String,
    /// Running heads (author on the left page, title on the right).
    pub heads: bool,
    /// Running heads' and folios' size.
    pub head_size: f32,
}

impl Default for Look {
    fn default() -> Self {
        Look {
            trim: (6.0, 9.0),
            margins: (0.75, 0.6, 0.75, 0.75),
            body_font: String::new(),
            heading_font: String::new(),
            body_size: 11.0,
            leading: 1.3,
            indent: 1.2,
            hyphenate: true,
            recto: true,
            sink: 1.8,
            number_size: 11.0,
            title_size: 20.0,
            ornament: String::new(),
            first_line: FirstLine::SmallCaps,
            break_mark: "*".into(),
            heads: true,
            head_size: 9.0,
        }
    }
}

/// Where a section starts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Start {
    /// The section's id.
    pub id: String,
    /// The page it starts on, from 1.
    pub page: usize,
}

/// A finished layout: its pages, its page map, and what can be made from it.
pub struct Typeset {
    doc: Document,
    layout: DocLayout,
    starts: Vec<(String, usize)>,
    title: String,
    author: String,
    date: String,
    passes: usize,
}

/// Lay out `book` with `look`. `date` (`YYYY-MM-DD`) is the date the PDF carries, from the
/// project, so the same book gives the same bytes.
///
/// Chapters that must open on a right-hand page get a blank page before them where they would
/// fall on a left-hand one; adding one moves everything after it, so the layout runs again until
/// nothing moves (at most [`MAX_PASSES`] times).
pub fn typeset(book: &PrintBook, look: &Look, date: &str) -> Result<Typeset, Error> {
    let mut blanks = std::collections::BTreeSet::new();
    let mut passes = 0;
    loop {
        passes += 1;
        let (doc, starts) = build::print_document(book, look, &blanks)?;
        let mut cache = LayoutCache::new();
        let layout = jcj_layout::print_layout(&doc, &mut cache);
        let mut t = Typeset { doc, layout, starts, title: book.title.clone(), author: book.author.clone(), date: date.to_owned(), passes };
        let mut wanted = std::collections::BTreeSet::new();
        if look.recto {
            // Each section starts on a new page, so a blank page added or removed before one moves
            // every later start by one: the shift so far predicts where each would land.
            let mut shift: i64 = 0;
            for (i, start) in t.page_map().iter().enumerate().skip(1) {
                let had = blanks.contains(&i);
                let without = start.page as i64 + shift - i64::from(had);
                let need = without % 2 == 0;
                if need {
                    wanted.insert(i);
                }
                shift += i64::from(need) - i64::from(had);
            }
        }
        if wanted == blanks || passes >= MAX_PASSES {
            t.passes = passes;
            return Ok(t);
        }
        blanks = wanted;
    }
}

/// The most layout passes one build runs.
pub const MAX_PASSES: usize = 4;

impl Typeset {
    /// How many pages.
    pub fn page_count(&self) -> usize {
        jcj_layout::page_count(&self.layout)
    }

    /// How many layout passes the build ran.
    pub fn passes(&self) -> usize {
        self.passes
    }

    /// The print document (for checks and the preview).
    pub fn document(&self) -> &Document {
        &self.doc
    }

    /// Page `index`'s size in points (width, height).
    pub fn page_size(&self, index: usize) -> Option<(f32, f32)> {
        jcj_layout::page_boxes(&self.layout).get(index).map(|b| (b.page.w, b.page.h))
    }

    /// Where each section starts, in reading order.
    pub fn page_map(&self) -> Vec<Start> {
        let pieces = jcj_layout::pages::body_pieces(&self.layout);
        self.starts
            .iter()
            .filter_map(|(id, block)| {
                let page = pieces.iter().position(|p| p.iter().any(|piece| piece.block == *block && !piece.continued))?;
                Some(Start { id: id.clone(), page: page + 1 })
            })
            .collect()
    }

    /// The PDF: every font embedded, tagged, no links, the project's date, naming the book.
    pub fn pdf(&self) -> Result<Vec<u8>, Error> {
        let settings = jcj_output::PdfSettings {
            title: Some(self.title.clone()),
            author: Some(self.author.clone()),
            created: Some(format!("{}T00:00:00", self.date)),
            tagged: true,
            compress: true,
            links: false,
            ..jcj_output::PdfSettings::default()
        };
        jcj_output::export_pdf(&self.doc, &self.layout, &settings).map_err(|e| Error::Pdf(format!("{e:?}")))
    }

    /// Page `index` (from 0) as a PNG at `ppi` pixels per inch, with its pixel size.
    pub fn page_png(&self, index: usize, ppi: f32) -> Result<(u32, u32, Vec<u8>), Error> {
        let img = jcj_output::render_page(&self.doc, &self.layout, index, ppi / INCH, &jcj_output::RenderSettings::default())
            .ok_or_else(|| Error::Page(format!("there is no page {}", index + 1)))?;
        Ok((img.width, img.height, img.to_png()))
    }
}
