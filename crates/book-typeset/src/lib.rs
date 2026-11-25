// SPDX-License-Identifier: MIT OR Apache-2.0
//! Print: a book and a design to pages, PDF, page images and the page map (plan section 5.6).
//!
//! Decision 0003: print is built on the core. A [`PrintBook`] (the book's sections and blocks in
//! reading order, made from the assembled book by [`from_book`]) and a [`Look`] (the design's
//! values for one edition) become a print document in the core's model (`jcj-doc`): a section per
//! opening, mirrored margins, running heads on facing pages and folios (roman in the front
//! matter), small caps, drop caps, hyphenation by the book's language, widow and orphan control,
//! endnotes per chapter, pictures with their alternative text, and the contents with page
//! numbers. The core lays it out (`jcj-layout`) and writes the PDF and page images
//! (`jcj-output`). Book's text is always data in the document's runs, never instructions.
//!
// core-candidate: open-source-core#19 (C5, a typesetting layer for books)

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

mod build;
pub mod fonts;
pub mod from_book;
pub mod screen;

use std::collections::BTreeSet;

use jcj_doc::Document;
use jcj_layout::{DocLayout, LayoutCache};

pub use build::{SOURCE, UNIT, print_document};

/// Points per inch.
pub const INCH: f32 = 72.0;

/// The name PDFs give as the program that made them.
pub const CREATOR: &str = "Wordsmith Book";

/// The most layout passes one build runs (right-hand starts, the contents' page numbers, the
/// gutter from the page count).
pub const MAX_PASSES: usize = 4;

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

/// A run of text with its marks. The text is exactly as given.
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
    /// Struck through.
    pub strike: bool,
    /// A language tag, for hyphenation.
    pub lang: Option<String>,
    /// Raised small (a note's number).
    pub superscript: bool,
}

impl Span {
    /// Plain text.
    pub fn plain(text: impl Into<String>) -> Span {
        Span { text: text.into(), ..Span::default() }
    }
}

/// A picture's place.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageFit {
    /// At its own size, no wider than the text.
    #[default]
    Inline,
    /// As wide as the text.
    Wide,
    /// On a page of its own, as large as the text block.
    Page,
}

/// A block of a section's text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    /// A body paragraph.
    Para(Vec<Span>),
    /// A scene break.
    Break,
    /// Verse: the lines of one stanza.
    Verse(Vec<Vec<Span>>),
    /// A block quote (or an aside) of paragraphs.
    Quote(Vec<Vec<Span>>),
    /// An aside of a kind (`letter`, `sign`, ...; empty for none): its paragraphs, set as the
    /// design says for that kind.
    Aside(String, Vec<Vec<Span>>),
    /// A subheading.
    Heading(Vec<Span>),
    /// A centered line of a front matter page, at `scale` times the body size (in hundredths).
    Centered(Vec<Span>, u16),
    /// A line of the contents: its text and the id of the section it points at.
    Entry(Vec<Span>, String),
    /// A picture: its bytes (PNG or JPEG), pixel size, alternative text and place.
    Image {
        /// The picture file.
        bytes: Vec<u8>,
        /// Its width and height in pixels.
        pixels: (u32, u32),
        /// Its alternative text (carried into the tagged PDF).
        alt: String,
        /// Its place.
        fit: ImageFit,
    },
}

/// What a section is, which decides how it opens.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kind {
    /// A chapter (or another body section): the designed opening.
    #[default]
    Chapter,
    /// A part page.
    Part,
    /// A front or back matter page with a title (a dedication, acknowledgements).
    Matter,
    /// A front matter page without a title (the half title, the title page, the copyright page).
    Plain,
}

/// One section that opens on a new page.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Section {
    /// Its stable id (for the page map).
    pub id: String,
    /// How it opens.
    pub kind: Kind,
    /// In the front matter (roman folios).
    pub front: bool,
    /// Opens on a right-hand page.
    pub recto: bool,
    /// The number line ("Chapter One").
    pub label: Option<String>,
    /// The title.
    pub title: String,
    /// A subtitle.
    pub subtitle: String,
    /// An epigraph and its attribution.
    pub epigraph: Option<(String, String)>,
    /// The text.
    pub blocks: Vec<Block>,
    /// Its notes, printed at its end, numbered from 1 (endnotes until footnotes arrive).
    pub notes: Vec<Vec<Span>>,
    /// The running head on right-hand pages when heads show chapters.
    pub head: String,
    /// The section's title is the book's own words (a generated page's), not the author's.
    pub generated_title: bool,
    /// The section's text is the book's own words (a generated page), not the author's.
    pub generated: bool,
    /// The book's Notes element: notes gathered at the book's end go here.
    pub holds_notes: bool,
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

/// Where a printed book's notes go (TXT-3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NotePlace {
    /// At the foot of the page that calls them, continued on the next page when long.
    Footnotes,
    /// At the end of each chapter.
    #[default]
    ChapterEnd,
    /// In a Notes section at the end of the book, under each chapter's name.
    BookEnd,
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

/// How a line of the opening is set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Case {
    /// As written.
    #[default]
    AsWritten,
    /// Small capitals.
    SmallCaps,
    /// Capitals.
    Capitals,
}

/// Running heads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Heads {
    /// The author on the left, the title on the right.
    #[default]
    AuthorTitle,
    /// The title on the left, the chapter on the right.
    TitleChapter,
    /// None.
    None,
}

/// The design's values for one edition, in points unless named otherwise.
#[derive(Clone, Debug, PartialEq)]
pub struct Look {
    /// Trim width and height, inches.
    pub trim: (f32, f32),
    /// Inside (gutter side), outside, top and bottom margins, inches.
    pub margins: (f32, f32, f32, f32),
    /// The gutter added to the inside margin by page count: (pages up to, inches).
    pub gutter: Vec<(usize, f32)>,
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
    /// Justified.
    pub justify: bool,
    /// Hyphenate.
    pub hyphenate: bool,
    /// Never two hyphenated lines in a row.
    pub strict_hyphens: bool,
    /// The opening's sink, a fraction of the text block.
    pub sink: f32,
    /// The number line's size.
    pub number_size: f32,
    /// The number line's case.
    pub number_case: Case,
    /// The title's size.
    pub title_size: f32,
    /// The title's case.
    pub title_case: Case,
    /// The opening centered (else at the left).
    pub center: bool,
    /// An ornament under the title ("" for none).
    pub ornament: String,
    /// The first line of each chapter.
    pub first_line: FirstLine,
    /// The scene break's mark.
    pub break_mark: String,
    /// A scene break is a blank line and the next scene starts in small capitals.
    pub break_space: bool,
    /// The part page's number size, title size and sink.
    pub part: (f32, f32, f32),
    /// Running heads.
    pub heads: Heads,
    /// Folios in the head (else at the foot).
    pub folio_head: bool,
    /// Heads' and folios' size.
    pub head_size: f32,
    /// Heads in small capitals.
    pub head_small_caps: bool,
    /// Folios on the pages (off for screens).
    pub folios: bool,
    /// Bleed past the trim on the outside, top and bottom, inches (0: none). The page is that
    /// much larger; the PDF says where the trim is.
    pub bleed: f32,
    /// Where the notes go.
    pub notes: NotePlace,
    /// How asides look, by kind.
    pub asides: wordsmith_book_design::design::Asides,
    /// Keep at least two lines of a paragraph at a page's foot and head (off only when the
    /// design allows single lines).
    pub widow_control: bool,
}

impl Default for Look {
    fn default() -> Self {
        Look {
            trim: (6.0, 9.0),
            margins: (0.75, 0.6, 0.75, 0.75),
            gutter: Vec::new(),
            body_font: String::new(),
            heading_font: String::new(),
            body_size: 11.0,
            leading: 1.3,
            indent: 1.2,
            justify: true,
            hyphenate: true,
            strict_hyphens: false,
            sink: 0.25,
            number_size: 11.0,
            number_case: Case::SmallCaps,
            title_size: 20.0,
            title_case: Case::AsWritten,
            center: true,
            ornament: String::new(),
            first_line: FirstLine::SmallCaps,
            break_mark: "*".into(),
            break_space: false,
            part: (14.0, 24.0, 0.33),
            heads: Heads::AuthorTitle,
            folio_head: false,
            head_size: 9.0,
            head_small_caps: true,
            folios: true,
            bleed: 0.0,
            notes: NotePlace::ChapterEnd,
            asides: wordsmith_book_design::design::Asides::default(),
            widow_control: true,
        }
    }
}

impl Look {
    /// The gutter for a book of `pages` pages, inches.
    pub fn gutter_for(&self, pages: usize) -> f32 {
        self.gutter.iter().find(|(upto, _)| pages <= *upto).or(self.gutter.last()).map_or(0.0, |(_, g)| *g)
    }
}

/// Where a section starts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Start {
    /// The section's id.
    pub id: String,
    /// The page it starts on, from 1 (the physical page).
    pub page: usize,
    /// The folio printed for that page ("vii", "12").
    pub folio: String,
}

/// The choices one layout pass is built with.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pass {
    /// Sections that get a blank page before them (to open on a right-hand page).
    pub blanks: BTreeSet<usize>,
    /// The contents' folios, by section id.
    pub folios: std::collections::BTreeMap<String, String>,
    /// The gutter, inches.
    pub gutter: f32,
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
    gutter: f32,
    /// The trim (inches) and bleed (inches), for the trim box.
    trim: (f32, f32),
    bleed: f32,
}

/// A paragraph's text as printed: objects left out, but a footnote reference shows its number.
fn shown_text(p: &jcj_doc::Paragraph) -> String {
    let marks: std::collections::BTreeMap<usize, String> = p
        .objects_with_offsets()
        .filter_map(|(at, o)| match o {
            jcj_doc::para::InlineObject::NoteRef { custom, .. } => Some((at, custom.clone())),
            _ => None,
        })
        .collect();
    let mut out = String::with_capacity(p.text.len());
    for (i, c) in p.text.char_indices() {
        if c == jcj_doc::para::OBJ {
            if let Some(m) = marks.get(&i) {
                out.push_str(m);
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// The book with every chapter's notes gathered into a Notes section at its end, under each
/// chapter's name, numbered as in the text.
pub fn notes_at_end(book: &PrintBook) -> PrintBook {
    let mut out = book.clone();
    let mut blocks = Vec::new();
    for s in &mut out.sections {
        if s.notes.is_empty() {
            continue;
        }
        let name = [s.label.clone().unwrap_or_default(), s.title.clone()].into_iter().filter(|x| !x.is_empty()).collect::<Vec<_>>().join(": ");
        blocks.push(Block::Heading(vec![Span::plain(if name.is_empty() { s.head.clone() } else { name })]));
        for (i, n) in std::mem::take(&mut s.notes).into_iter().enumerate() {
            let mut spans = vec![Span::plain(format!("{}. ", i + 1))];
            spans.extend(n);
            blocks.push(Block::Para(spans));
        }
    }
    if let Some(host) = out.sections.iter_mut().find(|s| s.holds_notes) {
        host.blocks.extend(blocks);
    } else if !blocks.is_empty() {
        out.sections.push(Section {
            id: "element:notes-at-end".into(),
            kind: Kind::Matter,
            recto: true,
            title: "Notes".into(),
            head: "Notes".into(),
            generated_title: true,
            blocks,
            ..Section::default()
        });
    }
    out
}

/// A typographic problem on a page (PRE-3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypoIssue {
    /// The page, from 1.
    pub page: usize,
    /// What it is.
    pub kind: TypoKind,
    /// The words where it is (empty for a river).
    pub text: String,
}

/// The kinds of typographic problem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypoKind {
    /// A paragraph's first line alone at a page's foot.
    Orphan,
    /// A paragraph's last line alone at a page's head.
    Widow,
    /// More hyphenated lines in a row than the design allows.
    StackedHyphens,
    /// A justified line with very wide word spaces.
    LooseLine,
    /// Word spaces lined up down the page.
    River,
}

/// Where a printed page starts, for an ebook's page list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageStart {
    /// The page's number, from 1.
    pub number: usize,
    /// Its folio as printed.
    pub label: String,
    /// The section (piece) it belongs to.
    pub section: String,
    /// The author's paragraph it starts in (its place in the section), when it starts in one.
    pub unit: Option<u32>,
    /// The byte in that paragraph's printed text where the page's first word starts.
    pub offset: usize,
}

/// `n` in lower-case roman numerals (front matter folios).
pub fn roman_lower(n: u32) -> String {
    const TABLE: [(u32, &str); 13] = [
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ];
    let mut n = n;
    let mut out = String::new();
    for (v, s) in TABLE {
        while n >= v {
            out.push_str(s);
            n -= v;
        }
    }
    out
}

/// A first guess at the page count, so the first pass starts with the gutter it will most
/// likely end with: the characters of the book over the characters a page holds, plus the pages
/// each section opens.
pub fn estimate_pages(book: &PrintBook, look: &Look) -> usize {
    let chars: usize = book
        .sections
        .iter()
        .flat_map(|s| &s.blocks)
        .map(|b| match b {
            Block::Para(sp) | Block::Heading(sp) | Block::Centered(sp, _) | Block::Entry(sp, _) => {
                sp.iter().map(|s| s.text.chars().count()).sum::<usize>() + 40
            }
            Block::Verse(l) | Block::Quote(l) | Block::Aside(_, l) => {
                l.iter().flatten().map(|s| s.text.chars().count()).sum::<usize>() + 80 * l.len()
            }
            Block::Break => 120,
            Block::Image { .. } => 1500,
        })
        .sum();
    let (w, h) = look.trim;
    let (i, o, t, b) = look.margins;
    let per_line = ((w - i - o - look.gutter_for(300)).max(1.0) * INCH / (look.body_size * 0.47)).max(10.0);
    let lines = ((h - t - b).max(1.0) * INCH / (look.body_size * look.leading)).max(5.0);
    let pages = chars as f32 / (per_line * lines);
    pages as usize + book.sections.len() * 3 / 2
}

/// Lay out `book` with `look`. `date` (`YYYY-MM-DD`) is the date the PDF carries, from the
/// project, so the same book gives the same bytes.
///
/// A book is laid out again until nothing moves (at most [`MAX_PASSES`] times): chapters that
/// must open on a right-hand page get a blank page before them, the contents get the folios
/// their sections landed on, and the gutter follows the page count.
pub fn typeset(book: &PrintBook, look: &Look, date: &str) -> Result<Typeset, Error> {
    let gathered;
    let book = if look.notes == NotePlace::BookEnd {
        gathered = notes_at_end(book);
        &gathered
    } else {
        book
    };
    let mut pass = Pass { gutter: look.gutter_for(estimate_pages(book, look)), ..Pass::default() };
    let mut n = 0;
    loop {
        n += 1;
        let (doc, starts) = build::print_document(book, look, &pass)?;
        let mut cache = LayoutCache::new();
        let layout = jcj_layout::print_layout(&doc, &mut cache);
        let t = Typeset {
            doc,
            layout,
            starts,
            title: book.title.clone(),
            author: book.author.clone(),
            date: date.to_owned(),
            passes: n,
            gutter: pass.gutter,
            trim: look.trim,
            bleed: look.bleed,
        };
        let map = t.page_map();
        let mut next = Pass { gutter: look.gutter_for(t.page_count()), ..Pass::default() };
        // Right-hand starts: each section starts on a new page, so a blank page added or removed
        // before one moves every later start by one.
        let mut shift: i64 = 0;
        for (i, (s, start)) in book.sections.iter().zip(&map).enumerate().skip(1) {
            let had = pass.blanks.contains(&i);
            let without = start.page as i64 + shift - i64::from(had);
            let need = s.recto && without % 2 == 0;
            if need {
                next.blanks.insert(i);
            }
            shift += i64::from(need) - i64::from(had);
        }
        next.folios = map.iter().map(|s| (s.id.clone(), s.folio.clone())).collect();
        if (next.blanks == pass.blanks && next.folios == pass.folios && (next.gutter - pass.gutter).abs() < 1e-4) || n >= MAX_PASSES {
            return Ok(t);
        }
        pass = next;
    }
}

impl Typeset {
    /// How many pages.
    pub fn page_count(&self) -> usize {
        jcj_layout::page_count(&self.layout)
    }

    /// How many layout passes the build ran.
    pub fn passes(&self) -> usize {
        self.passes
    }

    /// The gutter the build settled on, inches.
    pub fn gutter(&self) -> f32 {
        self.gutter
    }

    /// The print document (for checks and the preview).
    pub fn document(&self) -> &Document {
        &self.doc
    }

    /// The layout (for checks and the preview).
    pub fn layout(&self) -> &DocLayout {
        &self.layout
    }

    /// Page `index`'s size in points (width, height).
    pub fn page_size(&self, index: usize) -> Option<(f32, f32)> {
        jcj_layout::page_boxes(&self.layout).get(index).map(|b| (b.page.w, b.page.h))
    }

    /// Page `index`'s body area (inside its margins) in points: (x, y from the top, width,
    /// height).
    pub fn body_area(&self, index: usize) -> Option<(f32, f32, f32, f32)> {
        jcj_layout::page_boxes(&self.layout).get(index).map(|b| (b.body.x, b.body.y, b.body.w, b.body.h))
    }

    /// The folio for page `index` (from 0): roman in the front matter, else arabic.
    pub fn folio(&self, index: usize) -> String {
        let boxes = jcj_layout::page_boxes(&self.layout);
        let Some(b) = boxes.get(index) else { return String::new() };
        if build::FRONT.get_or_none(&self.doc).is_some_and(|front: Vec<usize>| front.contains(&b.section)) {
            roman_lower(b.number)
        } else {
            b.number.to_string()
        }
    }

    /// Where each section starts, in reading order.
    pub fn page_map(&self) -> Vec<Start> {
        let pieces = jcj_layout::pages::body_pieces(&self.layout);
        self.starts
            .iter()
            .filter_map(|(id, block)| {
                let page = pieces.iter().position(|p| p.iter().any(|piece| piece.block == *block && !piece.continued))?;
                Some(Start { id: id.clone(), page: page + 1, folio: self.folio(page) })
            })
            .collect()
    }

    /// Which section page `index` (from 0) belongs to: the last section starting at or before it.
    pub fn section_of_page(&self, index: usize) -> Option<String> {
        self.page_map().into_iter().rev().find(|s| s.page <= index + 1).map(|s| s.id)
    }

    /// The bleed, inches (0: none).
    pub fn bleed(&self) -> f32 {
        self.bleed
    }

    /// Page `index`'s trim box in points from the page's bottom left: the whole page without
    /// bleed; with bleed, the page less the bleed on the outside (right on right-hand pages, left
    /// on left-hand ones), top and bottom.
    pub fn trim_box(&self, index: usize) -> (f32, f32, f32, f32) {
        let b = self.bleed * INCH;
        let (w, h) = (self.trim.0 * INCH, self.trim.1 * INCH);
        let x0 = if index.is_multiple_of(2) { 0.0 } else { b };
        (x0, b, x0 + w, b + h)
    }

    /// The PDF: every font embedded, tagged, no links, the project's date, naming the book; with
    /// bleed, each page's trim and bleed boxes.
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
        let pdf = jcj_output::export_pdf(&self.doc, &self.layout, &settings).map_err(|e| Error::Pdf(format!("{e:?}")))?;
        if self.bleed <= 0.0 {
            return Ok(pdf);
        }
        let boxes: Vec<jcj_output::Boxes> = (0..self.page_count())
            .map(|i| {
                let (x0, y0, x1, y1) = self.trim_box(i);
                let (w, h) = self.page_size(i).unwrap_or((0.0, 0.0));
                let pb =
                    |a: f32, b: f32, c: f32, d: f32| jcj_output::PageBox { x0: f64::from(a), y0: f64::from(b), x1: f64::from(c), y1: f64::from(d) };
                jcj_output::Boxes { trim: pb(x0, y0, x1, y1), bleed: Some(pb(0.0, 0.0, w, h)) }
            })
            .collect();
        let limits = jcj_output::ReadLimits { max_input_bytes: 1 << 30, max_pages: 100_000, max_glyphs_per_page: 1 };
        jcj_output::set_page_boxes(&pdf, &boxes, &limits).map_err(|e| Error::Pdf(e.to_string()))
    }

    /// Page `index` (from 0) as a PNG at `ppi` pixels per inch, with its pixel size.
    pub fn page_png(&self, index: usize, ppi: f32) -> Result<(u32, u32, Vec<u8>), Error> {
        let img = self.page_image(index, ppi)?;
        Ok((img.width, img.height, img.to_png()))
    }

    /// Page `index` (from 0) as an RGBA image at `ppi` pixels per inch.
    pub fn page_image(&self, index: usize, ppi: f32) -> Result<jcj_output::PageImage, Error> {
        jcj_output::render_page(&self.doc, &self.layout, index, ppi / INCH, &jcj_output::RenderSettings::default())
            .ok_or_else(|| Error::Page(format!("there is no page {}", index + 1)))
    }

    /// Where each page starts, in reading order: its section, and when it starts in the author's
    /// text, the paragraph (its place in the section) and the byte where the page's first word is.
    pub fn page_starts(&self) -> Vec<PageStart> {
        let pieces = jcj_layout::pages::body_pieces(&self.layout);
        let body: Vec<&jcj_doc::Paragraph> = jcj_doc::walk::body_paragraphs(&self.doc).into_iter().map(|(_, p)| p).collect();
        (0..self.page_count())
            .map(|i| {
                let mut start =
                    PageStart { number: i + 1, label: self.folio(i), section: self.section_of_page(i).unwrap_or_default(), unit: None, offset: 0 };
                for piece in pieces.get(i).into_iter().flatten() {
                    let Some(p) = body.get(piece.block) else { continue };
                    let Some(unit) = UNIT.get_or_none(*p) else { continue };
                    let mut at = if piece.continued { piece.bytes.as_ref().map_or(0, |b| b.start) } else { 0 };
                    // The start of the word the page starts in.
                    while at > 0 && !p.text.is_char_boundary(at) {
                        at -= 1;
                    }
                    at = p.text[..at].rfind(char::is_whitespace).map_or(0, |w| w + 1);
                    start.unit = Some(unit);
                    start.offset = at;
                    break;
                }
                start
            })
            .collect()
    }

    /// The lines the author's paragraphs are set in, in order: each line's text (as in the
    /// paragraph, soft hyphens included) and whether it ends in a hyphen the layout added.
    pub fn author_lines(&self) -> Vec<(String, bool)> {
        let body: Vec<&jcj_doc::Paragraph> = jcj_doc::walk::body_paragraphs(&self.doc).into_iter().map(|(_, p)| p).collect();
        let mut out = Vec::new();
        for page in &self.layout.pages {
            for it in &page.items {
                let jcj_layout::Placed::Lines { story: jcj_doc::StoryRef::Body, path, para, l0, l1, .. } = it else { continue };
                let Some(p) = path.0.first().and_then(|b| body.get(*b as usize)) else { continue };
                if SOURCE.get_or_none(*p) != Some(true) {
                    continue;
                }
                for line in para.lines.iter().take(*l1).skip(*l0) {
                    let (Some(a), Some(b)) = (para.clusters.get(line.c0), line.c1.checked_sub(1).and_then(|c| para.clusters.get(c))) else {
                        continue;
                    };
                    out.push((p.text.get(a.start..b.end).unwrap_or_default().to_owned(), line.hyphen.is_some()));
                }
            }
        }
        out
    }

    /// Typographic problems by page (PRE-3): single lines of a paragraph at a page's foot
    /// (orphans) or head (widows), more hyphenated lines in a row than `hyphen_limit`, loose
    /// justified lines (a word space wider than `loose_em` of the type size), and rivers (word
    /// spaces lined up down four lines or more).
    pub fn typography(&self, hyphen_limit: usize, loose_em: f32) -> Vec<TypoIssue> {
        let mut out = Vec::new();
        let body: Vec<&jcj_doc::Paragraph> = jcj_doc::walk::body_paragraphs(&self.doc).into_iter().map(|(_, p)| p).collect();
        let pieces = jcj_layout::pages::body_pieces(&self.layout);
        let sample = |p: &jcj_doc::Paragraph, from: usize| -> String {
            let mut at = from.min(p.text.len());
            while !p.text.is_char_boundary(at) {
                at -= 1;
            }
            p.text[at..].replace(jcj_doc::para::OBJ, "").split_whitespace().take(6).collect::<Vec<_>>().join(" ")
        };
        for (i, ps) in pieces.iter().enumerate() {
            for piece in ps {
                let (Some(lines), Some(p)) = (&piece.lines, body.get(piece.block)) else { continue };
                if lines.end - lines.start != 1 || SOURCE.get_or_none(*p) != Some(true) {
                    continue;
                }
                let from = piece.bytes.as_ref().map_or(0, |b| b.start);
                if piece.continues {
                    out.push(TypoIssue { page: i + 1, kind: TypoKind::Orphan, text: sample(p, from) });
                }
                if piece.continued {
                    out.push(TypoIssue { page: i + 1, kind: TypoKind::Widow, text: sample(p, from) });
                }
            }
        }
        for (i, page) in self.layout.pages.iter().enumerate() {
            // Word spaces of each line on the page, as x positions on the page, for rivers.
            let mut rows: Vec<(f32, Vec<f32>)> = Vec::new();
            for it in &page.items {
                let jcj_layout::Placed::Lines { story: jcj_doc::StoryRef::Body, path, para, l0, l1, x, y, .. } = it else { continue };
                let Some(p) = path.0.first().and_then(|b| body.get(*b as usize)) else { continue };
                if SOURCE.get_or_none(*p) != Some(true) {
                    continue;
                }
                let size = p.runs.first().and_then(|r| r.props.size).unwrap_or(11.0);
                let mut run = 0;
                let last = para.lines.len().saturating_sub(1);
                for (n, line) in para.lines.iter().enumerate().take(*l1).skip(*l0) {
                    run = if line.hyphen.is_some() { run + 1 } else { 0 };
                    if run == hyphen_limit + 1 {
                        out.push(TypoIssue { page: i + 1, kind: TypoKind::StackedHyphens, text: sample(p, line.start) });
                    }
                    let mut spaces = Vec::new();
                    for c in line.c0..line.c1 {
                        let Some(cl) = para.clusters.get(c) else { continue };
                        if p.text.get(cl.start..cl.end) != Some(" ") {
                            continue;
                        }
                        let k = c - line.c0;
                        let (Some(a), Some(b)) = (line.xs.get(k), line.xs.get(k + 1)) else { continue };
                        spaces.push((x + (a + b) / 2.0, b - a));
                    }
                    // A justified line's widest space (the last line of a paragraph is not
                    // stretched).
                    if n != last && spaces.iter().any(|s| s.1 > loose_em * size) {
                        out.push(TypoIssue { page: i + 1, kind: TypoKind::LooseLine, text: sample(p, line.start) });
                    }
                    rows.push((y + line.top, spaces.iter().map(|s| s.0).collect()));
                }
            }
            rows.sort_by(|a, b| a.0.total_cmp(&b.0));
            // A river: a space within a point and a half of one on the line above, four lines on.
            let mut chains: Vec<(f32, usize)> = Vec::new();
            let mut reported = false;
            for (_, xs) in &rows {
                let next: Vec<(f32, usize)> =
                    xs.iter().map(|x| (*x, chains.iter().filter(|(c, _)| (c - x).abs() <= 1.5).map(|(_, n)| n + 1).max().unwrap_or(1))).collect();
                if !reported && next.iter().any(|(_, n)| *n >= 4) {
                    out.push(TypoIssue { page: i + 1, kind: TypoKind::River, text: String::new() });
                    reported = true;
                }
                chains = next;
            }
        }
        out.sort_by_key(|t| t.page);
        out
    }

    /// Where page `index`'s footnotes begin, in points from the top, if it has any.
    pub fn notes_top(&self, index: usize) -> Option<f32> {
        let page = self.layout.pages.get(index)?;
        page.items
            .iter()
            .filter_map(|it| match it {
                jcj_layout::Placed::Lines { story: jcj_doc::StoryRef::Part(id), y, .. }
                    if self.doc.parts.get(id).is_some_and(|p| p.kind == jcj_doc::PartKind::Footnote) =>
                {
                    Some(*y)
                }
                _ => None,
            })
            .reduce(f32::min)
    }

    /// The footnotes' text, in the order the book calls them.
    pub fn printed_notes(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (_, p) in jcj_doc::walk::body_paragraphs(&self.doc) {
            for (_, obj) in p.objects_with_offsets() {
                if let jcj_doc::para::InlineObject::NoteRef { kind: jcj_doc::para::NoteKind::Footnote, id, .. } = obj
                    && let Some(part) = self.doc.parts.get(id)
                {
                    for b in &part.blocks {
                        if let jcj_doc::Block::Para(np) = b.as_ref() {
                            out.push(np.text.replace(jcj_doc::para::OBJ, ""));
                        }
                    }
                }
            }
        }
        out
    }

    /// The smallest type the book's text is set in (points), with some of the text in it: every
    /// run of the body that prints letters, and the running heads and folios. Superscript note
    /// numbers are left out (print services measure the text, not its marks).
    pub fn smallest_type(&self, look: &Look) -> Option<(f32, String)> {
        let mut best: Option<(f32, String)> = None;
        let mut consider = |size: f32, text: &str| {
            if best.as_ref().is_none_or(|(b, _)| size < *b) {
                best = Some((size, text.chars().take(40).collect()));
            }
        };
        let default = self.doc.styles.default_chr.size.unwrap_or(11.0);
        for (_, p) in jcj_doc::walk::body_paragraphs(&self.doc) {
            let mut at = 0;
            for r in &p.runs {
                let text: String = p.text.get(at..at + r.len).unwrap_or("").to_owned();
                at += r.len;
                if r.props.vert_align == Some(jcj_doc::props::VertAlign::Superscript) || !text.chars().any(char::is_alphanumeric) {
                    continue;
                }
                consider(r.props.size.or(p.mark.size).unwrap_or(default), &text);
            }
        }
        if look.heads != Heads::None || look.folios {
            consider(look.head_size, "running heads and page numbers");
        }
        best
    }

    /// The text the book's body area prints, paragraph by paragraph: (the paragraph's text,
    /// whether it is the author's words).
    pub fn printed_paragraphs(&self) -> Vec<(String, bool)> {
        jcj_doc::walk::body_paragraphs(&self.doc).into_iter().map(|(_, p)| (shown_text(p), SOURCE.get_or_none(p).unwrap_or(false))).collect()
    }
}
