// SPDX-License-Identifier: MIT OR Apache-2.0
//! From the assembled book, a design and an edition to what print lays out: a [`PrintBook`] and a
//! [`Look`]. Every block and mark of the book view is carried: paragraphs with italic, bold,
//! small caps, strikethrough and language spans; headings; scene breaks; block quotes; verse with
//! its lines; asides (as block quotes until R2); pictures with their alternative text; notes,
//! numbered per chapter and printed at the chapter's end. Raw blocks for other formats are left
//! out and counted.

use wordsmith_book_design::Design;
use wordsmith_book_design::design::{self, BreakStyle};
use wordsmith_book_design::fonts::Role;
use wordsmith_book_format::assemble::{Assembled, Piece, What};
use wordsmith_book_model::{ElementKind, Only};
use wordsmith_manuscript_format::book::FrontKind;
use wordsmith_manuscript_model::{Block as MBlock, Fence, ImageRole, Inline, Inlines, Note};

use crate::{Block, Case, Error, FirstLine, Heads, ImageFit, Kind, Look, PrintBook, Section, Span};

/// The page's geometry for an edition: trim and margins, inches, and the gutter by page count.
#[derive(Clone, Debug, PartialEq)]
pub struct Geometry {
    /// Trim width and height.
    pub trim: (f32, f32),
    /// Inside, outside, top and bottom margins (the inside before the gutter).
    pub margins: (f32, f32, f32, f32),
    /// The gutter by page count: (pages up to, inches).
    pub gutter: Vec<(usize, f32)>,
    /// Bleed on the outside, top and bottom, inches (0: none).
    pub bleed: f32,
    /// Where the notes go.
    pub notes: crate::NotePlace,
}

impl Default for Geometry {
    fn default() -> Self {
        Geometry {
            trim: (6.0, 9.0),
            margins: (0.5, 0.6, 0.75, 0.75),
            gutter: vec![(150, 0.375), (300, 0.5), (500, 0.625), (700, 0.75), (usize::MAX, 0.875)],
            bleed: 0.0,
            notes: crate::NotePlace::ChapterEnd,
        }
    }
}

/// A trim id (`6x9`, `5.5x8.5`) as inches.
pub fn parse_trim(id: &str) -> Option<(f32, f32)> {
    let (w, h) = id.split_once(['x', '×'])?;
    let (w, h) = (w.trim().parse::<f32>().ok()?, h.trim().parse::<f32>().ok()?);
    (w > 2.0 && h > 2.0 && w < 20.0 && h < 20.0).then_some((w, h))
}

fn case(c: design::Case) -> Case {
    match c {
        design::Case::AsWritten => Case::AsWritten,
        design::Case::SmallCaps => Case::SmallCaps,
        design::Case::Capitals => Case::Capitals,
    }
}

fn family(role: &str) -> Result<String, Error> {
    let r = Role::from_key(role).ok_or_else(|| Error::Setup(format!("the design names the font role “{role}”, which Book does not have")))?;
    crate::fonts::family(r).ok_or_else(|| Error::Setup(format!("the {} font could not be loaded", r.title().to_lowercase())))
}

/// The look of `design` on `geometry`.
pub fn look(design: &Design, geometry: &Geometry) -> Result<Look, Error> {
    let d = design;
    Ok(Look {
        trim: geometry.trim,
        margins: geometry.margins,
        gutter: geometry.gutter.clone(),
        body_font: family(&d.body.font)?,
        heading_font: family(&d.chapter.font)?,
        body_size: d.body.size,
        leading: d.body.leading,
        indent: d.body.indent,
        justify: d.body.justify,
        hyphenate: d.body.hyphenation != design::Hyphenation::Off,
        strict_hyphens: d.body.hyphenation == design::Hyphenation::Strict,
        sink: d.chapter.sink,
        number_size: d.chapter.number_size,
        number_case: case(d.chapter.number_case),
        title_size: d.chapter.title_size,
        title_case: case(d.chapter.title_case),
        center: d.chapter.align == design::Align::Center,
        ornament: d.chapter.ornament.clone(),
        first_line: match d.chapter.first_line {
            design::FirstLine::None => FirstLine::Plain,
            design::FirstLine::SmallCaps => FirstLine::SmallCaps,
            design::FirstLine::DropCap => FirstLine::DropCap(d.chapter.drop_lines),
        },
        break_mark: d.scene_break.mark.clone(),
        break_space: d.scene_break.style == BreakStyle::Space,
        part: (d.part.number_size, d.part.title_size, d.part.sink),
        heads: match d.heads.style {
            design::Heads::AuthorTitle => Heads::AuthorTitle,
            design::Heads::TitleChapter => Heads::TitleChapter,
            design::Heads::None => Heads::None,
        },
        folio_head: d.heads.folio == design::Folio::Head,
        head_size: d.heads.size,
        head_small_caps: d.heads.small_caps,
        folios: true,
        bleed: geometry.bleed,
        notes: geometry.notes,
        asides: d.asides.clone(),
    })
}

pub use wordsmith_book_model::picture::size as image_size;

/// What converting a piece left out, for the build report.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Left {
    /// Raw blocks for other formats.
    pub raw: usize,
    /// Pictures whose file was missing or not a PNG or JPEG.
    pub pictures: Vec<String>,
}

struct Convert<'a> {
    lang: &'a str,
    images: &'a dyn Fn(&str) -> Option<Vec<u8>>,
    left: Left,
}

impl Convert<'_> {
    fn spans(&self, inlines: &Inlines, labels: &[String]) -> Vec<Span> {
        let mut out = Vec::new();
        for i in inlines {
            match i {
                Inline::Span { text, marks } => out.push(Span {
                    text: text.clone(),
                    italic: marks.italic,
                    bold: marks.bold,
                    small_caps: marks.small_caps,
                    strike: marks.strike,
                    lang: marks.lang.clone().filter(|l| l != self.lang),
                    superscript: false,
                }),
                Inline::NoteRef { label, .. } => {
                    let n = labels.iter().position(|l| l == label).map_or(0, |p| p + 1);
                    if n > 0 {
                        out.push(Span { text: n.to_string(), superscript: true, ..Span::default() });
                    }
                }
            }
        }
        out
    }

    fn blocks(&mut self, blocks: &[MBlock], labels: &[String]) -> Vec<Block> {
        let mut out = Vec::new();
        for b in blocks {
            match b {
                MBlock::Para(c) => out.push(Block::Para(self.spans(c, labels))),
                MBlock::Heading { content, .. } => out.push(Block::Heading(self.spans(content, labels))),
                MBlock::SceneBreak => out.push(Block::Break),
                MBlock::Quote(ps) => out.push(Block::Quote(ps.iter().map(|p| self.spans(p, labels)).collect())),
                MBlock::Fenced { kind: Fence::Aside(kind), paras: ps } => {
                    out.push(Block::Aside(kind.clone().unwrap_or_default(), ps.iter().map(|p| self.spans(p, labels)).collect()))
                }
                MBlock::Fenced { kind: Fence::Verse, paras } => {
                    for stanza in paras {
                        let spans = self.spans(stanza, labels);
                        let mut lines: Vec<Vec<Span>> = vec![Vec::new()];
                        for s in spans {
                            let mut parts = s.text.split('\n').peekable();
                            while let Some(part) = parts.next() {
                                if !part.is_empty()
                                    && let Some(line) = lines.last_mut()
                                {
                                    line.push(Span { text: part.to_owned(), ..s.clone() });
                                }
                                if parts.peek().is_some() {
                                    lines.push(Vec::new());
                                }
                            }
                        }
                        out.push(Block::Verse(lines));
                    }
                }
                MBlock::Image { alt, src, role } => match (self.images)(src).and_then(|bytes| image_size(&bytes).map(|px| (bytes, px))) {
                    Some((bytes, pixels)) => out.push(Block::Image {
                        bytes,
                        pixels,
                        alt: alt.clone(),
                        fit: match role {
                            ImageRole::Inline => ImageFit::Inline,
                            ImageRole::Wide => ImageFit::Wide,
                            ImageRole::Page => ImageFit::Page,
                        },
                    }),
                    None => self.left.pictures.push(src.clone()),
                },
                MBlock::Raw { .. } => self.left.raw += 1,
            }
        }
        out
    }

    fn notes(&self, notes: &[Note]) -> Vec<Vec<Span>> {
        notes
            .iter()
            .map(|n| {
                let mut spans = Vec::new();
                for (i, p) in n.paras.iter().enumerate() {
                    if i > 0 {
                        spans.push(Span::plain(" "));
                    }
                    spans.extend(self.spans(p, &[]));
                }
                spans
            })
            .collect()
    }
}

fn element_blocks(kind: ElementKind, blocks: &[MBlock]) -> Vec<Block> {
    let spans =
        |c: &Inlines| c.iter().filter_map(|i| if let Inline::Span { text, .. } = i { Some(Span::plain(text)) } else { None }).collect::<Vec<_>>();
    blocks
        .iter()
        .map(|b| match (kind, b) {
            (_, MBlock::Heading { level: 1, content }) => Block::Centered(spans(content), 220),
            (_, MBlock::Heading { level: 2, content }) => Block::Centered(spans(content), 140),
            (_, MBlock::Heading { content, .. }) => Block::Centered(spans(content), 125),
            (ElementKind::Copyright | ElementKind::TitlePage | ElementKind::HalfTitle, MBlock::Para(c)) => Block::Centered(spans(c), 85),
            (ElementKind::Dedication | ElementKind::Epigraph, MBlock::Para(c)) => Block::Centered(spans(c), 100),
            (_, MBlock::Para(c)) => Block::Para(spans(c)),
            _ => Block::Para(Vec::new()),
        })
        .collect()
}

fn label_and_title(p: &Piece) -> String {
    [p.label.clone().unwrap_or_default(), p.title.clone()].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(": ")
}

/// Whether a piece is printed in a print edition.
fn printed(only: Only) -> bool {
    only != Only::Ebook
}

/// What print lays out for an assembled book with a design on a page geometry. `images` gives a
/// picture's bytes by its path in the source. Gives the print book, its look and what was left
/// out.
pub fn print_book(
    a: &Assembled,
    d: &Design,
    geometry: &Geometry,
    lang: &str,
    images: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Result<(PrintBook, Look, Left), Error> {
    let look = look(d, geometry)?;
    let recto_chapters = d.chapter.start == design::Start::Recto;
    let mut c = Convert { lang, images, left: Left::default() };
    let mut sections = Vec::new();
    let contents: Vec<Block> = a
        .body
        .iter()
        .chain(&a.back)
        .filter(|p| {
            printed(p.only) && (matches!(p.what, What::Part | What::Chapter | What::Other) || matches!(p.what, What::Matter(_) | What::Element(_)))
        })
        .filter(|p| !matches!(p.what, What::Element(ElementKind::Contents)))
        .map(|p| {
            let mut spans = vec![Span::plain(label_and_title(p))];
            if p.what == What::Part {
                spans[0].small_caps = true;
            }
            Block::Entry(spans, p.id.clone())
        })
        .collect();
    for (front, pieces) in [(true, &a.front), (false, &a.body), (false, &a.back)] {
        for p in pieces.iter().filter(|p| printed(p.only)) {
            let labels: Vec<String> = p.notes.iter().map(|n| n.label.clone()).collect();
            let mut s = Section {
                id: p.id.clone(),
                front,
                label: p.label.clone(),
                title: p.title.clone(),
                subtitle: p.subtitle.clone(),
                epigraph: p.epigraph.clone(),
                head: if p.title.is_empty() { p.label.clone().unwrap_or_default() } else { p.title.clone() },
                ..Section::default()
            };
            match p.what {
                What::Element(ElementKind::Contents) => {
                    s.kind = Kind::Matter;
                    s.recto = true;
                    s.generated = true;
                    s.generated_title = true;
                    s.blocks = contents.clone();
                }
                What::Element(kind @ (ElementKind::HalfTitle | ElementKind::TitlePage | ElementKind::Copyright)) => {
                    s.kind = Kind::Plain;
                    s.title.clear();
                    s.recto = kind != ElementKind::Copyright;
                    s.generated = true;
                    s.generated_title = true;
                    s.blocks = element_blocks(kind, &p.blocks);
                }
                What::Element(kind) => {
                    s.kind = if matches!(kind, ElementKind::Dedication | ElementKind::Epigraph) { Kind::Plain } else { Kind::Matter };
                    if s.kind == Kind::Plain {
                        s.title.clear();
                    }
                    s.recto = front;
                    s.generated = true;
                    s.generated_title = true;
                    s.blocks = element_blocks(kind, &p.blocks);
                }
                What::Matter(kind) => {
                    s.kind = if matches!(kind, FrontKind::TitlePage | FrontKind::Copyright | FrontKind::Dedication | FrontKind::Epigraph) {
                        Kind::Plain
                    } else {
                        Kind::Matter
                    };
                    if s.kind == Kind::Plain {
                        s.title.clear();
                    }
                    s.recto = front && kind != FrontKind::Copyright;
                    // The book view's words leave front and back matter titles out.
                    s.generated_title = true;
                    s.blocks = c.blocks(&p.blocks, &labels);
                    // A plain page (the writer's title page, copyright, dedication) is centered.
                    if s.kind == Kind::Plain {
                        s.blocks = s
                            .blocks
                            .into_iter()
                            .map(|b| match b {
                                Block::Para(sp) => Block::Centered(sp, if kind == FrontKind::TitlePage { 160 } else { 95 }),
                                Block::Heading(sp) => Block::Centered(sp, 160),
                                other => other,
                            })
                            .collect();
                    }
                    s.notes = c.notes(&p.notes);
                }
                What::Part => {
                    s.kind = Kind::Part;
                    s.recto = true;
                    s.blocks = c.blocks(&p.blocks, &labels);
                    s.notes = c.notes(&p.notes);
                }
                What::Chapter | What::Other => {
                    s.kind = Kind::Chapter;
                    s.recto = recto_chapters;
                    s.generated_title = p.what == What::Other;
                    s.blocks = c.blocks(&p.blocks, &labels);
                    s.notes = c.notes(&p.notes);
                }
            }
            sections.push(s);
        }
    }
    let book = PrintBook { title: a.meta.title.clone(), author: a.meta.byline.clone(), lang: lang.to_owned(), sections };
    Ok((book, look, c.left))
}

/// The author's words a print book holds, in reading order: each section's title when it is the
/// author's, its text, then its notes (what the printed source paragraphs must match).
pub fn source_words(book: &PrintBook) -> Vec<String> {
    let mut out = Vec::new();
    let words = |spans: &[Span], out: &mut Vec<String>| {
        let t: String = spans.iter().filter(|s| !s.superscript).map(|s| s.text.as_str()).collect();
        out.extend(jcj_features_words(&t));
    };
    for s in &book.sections {
        if s.generated {
            continue;
        }
        if !s.generated_title {
            out.extend(jcj_features_words(&s.title));
        }
        for b in &s.blocks {
            match b {
                Block::Para(sp) | Block::Heading(sp) | Block::Centered(sp, _) | Block::Entry(sp, _) => words(sp, &mut out),
                Block::Verse(lines) => {
                    let t: Vec<Span> = lines
                        .iter()
                        .enumerate()
                        .flat_map(|(i, l)| if i > 0 { vec![Span::plain("\n")] } else { vec![] }.into_iter().chain(l.iter().cloned()))
                        .collect();
                    words(&t, &mut out);
                }
                Block::Quote(ps) | Block::Aside(_, ps) => ps.iter().for_each(|p| words(p, &mut out)),
                Block::Break | Block::Image { .. } => {}
            }
        }
        for n in &s.notes {
            words(n, &mut out);
        }
    }
    out
}

/// Whitespace-separated words holding a letter or digit (the core's word rule).
fn jcj_features_words(text: &str) -> Vec<String> {
    text.split(char::is_whitespace).filter(|w| w.chars().any(char::is_alphanumeric)).map(str::to_owned).collect()
}
