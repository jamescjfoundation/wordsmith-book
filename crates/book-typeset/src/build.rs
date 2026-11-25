// SPDX-License-Identifier: MIT OR Apache-2.0
//! A [`PrintBook`] and a [`Look`] to a print document in the core's model.

use std::collections::BTreeMap;
use std::sync::Arc;

use jcj_doc::para::{InlineObject, OBJ};
use jcj_doc::props::{Align, CharProps, LineSpacing, ParaProps, TabAlign, TabLeader, TabStop, VertAlign};
use jcj_doc::section::{NumFormat, SectionStart};
use jcj_doc::{Block as DocBlock, Document, ExtKey, Paragraph, PartKind, SectionProps};

use crate::{Block, Case, Error, FirstLine, Heads, INCH, ImageFit, Kind, Look, Pass, PrintBook, Section, Span};

/// On a paragraph: it holds the author's words (the word check compares these with the source).
pub const SOURCE: ExtKey<bool> = ExtKey::new("book.source");
/// On a run: the book's own words inside an author's paragraph (a note's number).
pub const GENERATED: ExtKey<bool> = ExtKey::new("book.generated");
/// On the document: the indexes of its sections that are front matter (roman folios).
pub const FRONT: ExtKey<Vec<usize>> = ExtKey::new("book.front_sections");

/// The line break inside a paragraph (verse).
const LINE_BREAK: char = '\n';

struct Builder<'a> {
    look: &'a Look,
    lang: String,
    pass: &'a Pass,
    body: Vec<Paragraph>,
    media: BTreeMap<String, Arc<Vec<u8>>>,
    /// Paragraphs (by index) that are full-page pictures: what follows starts a new page.
    alone: Vec<usize>,
    text_width: f32,
    text_height: f32,
}

fn page_field() -> InlineObject {
    InlineObject::Field { instr: "PAGE".into(), result: String::new(), locked: false, dirty: false, code: false, form: String::new() }
}

impl Builder<'_> {
    fn chars(&self, font: &str, size: f32) -> CharProps {
        CharProps { font: Some(font.to_owned()), size: Some(size), lang: Some(self.lang.clone()), ..CharProps::default() }
    }

    fn body_chars(&self) -> CharProps {
        self.chars(&self.look.body_font, self.look.body_size)
    }

    fn line(&self) -> f32 {
        self.look.body_size * self.look.leading
    }

    fn body_props(&self, indent: bool) -> ParaProps {
        ParaProps {
            align: Some(if self.look.justify { Align::Justify } else { Align::Left }),
            indent_first: Some(if indent { self.look.indent * self.look.body_size } else { 0.0 }),
            line_spacing: Some(LineSpacing::Exactly(self.line())),
            space_before: Some(0.0),
            space_after: Some(0.0),
            widow_control: Some(true),
            ..ParaProps::default()
        }
    }

    fn centered(&self, before: f32, after: f32) -> ParaProps {
        ParaProps {
            align: Some(if self.look.center { Align::Center } else { Align::Left }),
            indent_first: Some(0.0),
            space_before: Some(before),
            space_after: Some(after),
            keep_next: Some(true),
            line_spacing: Some(LineSpacing::Multiple(1.0)),
            ..ParaProps::default()
        }
    }

    fn para(&self, spans: &[Span], props: ParaProps, base: &CharProps, source: bool) -> Paragraph {
        let mut p = Paragraph::new();
        p.props = props;
        p.mark = base.clone();
        for s in spans {
            let mut c = base.clone();
            if s.italic {
                c.italic = Some(true);
            }
            if s.bold {
                c.bold = Some(true);
            }
            if s.small_caps {
                c.small_caps = Some(true);
            }
            if s.strike {
                c.strike = Some(true);
            }
            if let Some(l) = &s.lang {
                c.lang = Some(l.clone());
            }
            if s.superscript {
                c.vert_align = Some(VertAlign::Superscript);
                let _ = GENERATED.set(&mut c, &true);
            }
            let at = p.text.len();
            let text = s.text.replace(OBJ, "");
            let _ = p.insert_text(at, &text, &c);
        }
        if source {
            let _ = SOURCE.set(&mut p, &true);
        }
        p
    }

    fn cased(&self, base: &CharProps, case: Case) -> CharProps {
        let mut c = base.clone();
        match case {
            Case::AsWritten => {}
            Case::SmallCaps => {
                c.small_caps = Some(true);
                c.spacing = Some(1.2);
            }
            Case::Capitals => {
                c.caps = Some(true);
                c.spacing = Some(0.8);
            }
        }
        c
    }

    fn lead_in(&self, spans: &[Span]) -> Vec<Span> {
        let mut out = spans.to_vec();
        let Some(first) = out.first_mut() else { return out };
        let mut cut = first.text.find(' ').unwrap_or(first.text.len());
        // A short first word takes the next word with it ("I had", "A man").
        if first.text[..cut].chars().count() < 3
            && let Some(next) = first.text.get(cut + 1..).and_then(|rest| rest.find(' '))
        {
            cut += 1 + next;
        }
        let rest = first.text.split_off(cut.min(first.text.len()));
        first.small_caps = true;
        let tail = Span { text: rest, small_caps: false, ..first.clone() };
        if !tail.text.is_empty() {
            out.insert(1, tail);
        }
        out
    }

    fn sink(&self, fraction: f32) -> f32 {
        (self.text_height * fraction).max(0.0)
    }

    fn opening(&mut self, s: &Section) {
        let look = self.look;
        let heading = self.chars(&look.heading_font, look.number_size);
        let mut first = true;
        let mut before = |this: &Self, sink: f32| -> f32 {
            if first {
                first = false;
                this.sink(sink)
            } else {
                0.0
            }
        };
        let (sink, number_size, title_size) = match s.kind {
            Kind::Part => (look.part.2, look.part.0, look.part.1),
            Kind::Matter => (look.sink * 0.6, look.number_size, look.title_size * 0.8),
            _ => (look.sink, look.number_size, look.title_size),
        };
        if let Some(label) = &s.label {
            let mut c = self.cased(&heading, look.number_case);
            c.size = Some(number_size);
            let b = before(self, sink);
            self.body.push(self.para(&[Span::plain(label)], self.centered(b, self.line() * 0.6), &c, false));
        }
        if !s.title.is_empty() {
            let c = self.cased(&self.chars(&look.heading_font, title_size), look.title_case);
            let b = before(self, sink);
            self.body.push(self.para(&[Span::plain(&s.title)], self.centered(b, self.line() * 0.4), &c, !s.generated_title));
        }
        if !s.subtitle.is_empty() {
            let mut c = self.chars(&look.body_font, look.body_size * 1.15);
            c.italic = Some(true);
            let b = before(self, sink);
            self.body.push(self.para(&[Span::plain(&s.subtitle)], self.centered(b, self.line() * 0.4), &c, false));
        }
        if !look.ornament.is_empty() && matches!(s.kind, Kind::Chapter | Kind::Part) {
            let c = self.chars(&look.body_font, look.title_size * 0.7);
            let b = before(self, sink);
            self.body.push(self.para(&[Span::plain(&look.ornament)], self.centered(b, self.line() * 0.4), &c, false));
        }
        if let Some((text, by)) = &s.epigraph {
            let mut c = self.chars(&look.body_font, look.body_size * 0.92);
            c.italic = Some(true);
            let indent = look.body_size * 3.0;
            let props = |this: &Self, before: f32| ParaProps {
                align: Some(Align::Left),
                indent_left: Some(indent),
                indent_right: Some(indent),
                indent_first: Some(0.0),
                space_before: Some(before),
                space_after: Some(0.0),
                line_spacing: Some(LineSpacing::Exactly(this.line() * 0.95)),
                keep_next: Some(true),
                ..ParaProps::default()
            };
            let b = before(self, sink).max(self.line());
            self.body.push(self.para(&[Span::plain(text)], props(self, b), &c, false));
            if !by.is_empty() {
                let mut p = props(self, 0.0);
                p.align = Some(Align::Right);
                let mut c2 = c.clone();
                c2.italic = None;
                self.body.push(self.para(&[Span::plain(format!("— {by}"))], p, &c2, false));
            }
        }
        if first {
            // An opening without a title still sinks.
            let b = self.sink(sink);
            let mut p = self.centered(b, 0.0);
            p.line_spacing = Some(LineSpacing::Exactly(1.0));
            self.body.push(self.para(&[], p, &self.body_chars(), false));
        } else {
            let mut p = self.centered(0.0, self.line());
            p.line_spacing = Some(LineSpacing::Exactly(1.0));
            self.body.push(self.para(&[], p, &self.body_chars(), false));
        }
    }

    fn image(&mut self, bytes: &[u8], pixels: (u32, u32), alt: &str, fit: ImageFit, source: bool) {
        let key = format!("image-{}.{}", self.media.len() + 1, if bytes.starts_with(b"\x89PNG") { "png" } else { "jpg" });
        self.media.insert(key.clone(), Arc::new(bytes.to_vec()));
        let (pw, ph) = (pixels.0.max(1) as f32, pixels.1.max(1) as f32);
        let natural = (pw / 300.0 * INCH, ph / 300.0 * INCH);
        let (w, h) = match fit {
            ImageFit::Inline => {
                let w = natural.0.min(self.text_width);
                (w, w * ph / pw)
            }
            ImageFit::Wide => (self.text_width, self.text_width * ph / pw),
            ImageFit::Page => {
                let scale = (self.text_width / pw).min(self.text_height * 0.95 / ph);
                (pw * scale, ph * scale)
            }
        };
        // With bleed, a full-page picture fills the whole page to the cut and past it: anchored to
        // the page behind the text, cropped to the page's shape.
        let bleeds = fit == ImageFit::Page && self.look.bleed > 0.0;
        let (w, h, float, crop) = if bleeds {
            let (pw_pt, ph_pt) = ((self.look.trim.0 + self.look.bleed) * INCH, (self.look.trim.1 + 2.0 * self.look.bleed) * INCH);
            let (page_aspect, image_aspect) = (pw_pt / ph_pt, pw / ph);
            let crop = if image_aspect > page_aspect {
                let side = (1.0 - page_aspect / image_aspect) / 2.0;
                [side, 0.0, side, 0.0]
            } else {
                let side = (1.0 - image_aspect / page_aspect) / 2.0;
                [0.0, side, 0.0, side]
            };
            let float = jcj_doc::para::Float {
                wrap: jcj_doc::para::Wrap::BehindText,
                h_rel: jcj_doc::para::Anchor::Page,
                v_rel: jcj_doc::para::Anchor::Page,
                ..Default::default()
            };
            (pw_pt, ph_pt, float, crop)
        } else {
            (w, h, jcj_doc::para::Float::default(), [0.0; 4])
        };
        let obj = InlineObject::Image { media: key, w, h, alt: alt.to_owned(), float, crop, fx: Default::default(), ole: None };
        let mut p = Paragraph::new();
        p.props = ParaProps {
            align: Some(Align::Center),
            space_before: Some(self.line() * 0.5),
            page_break_before: Some(fit == ImageFit::Page),
            space_after: Some(self.line() * 0.5),
            line_spacing: Some(LineSpacing::Multiple(1.0)),
            ..ParaProps::default()
        };
        let _ = p.insert_object(0, obj, &self.body_chars());
        if source {
            let _ = SOURCE.set(&mut p, &true);
        }
        if fit == ImageFit::Page {
            // A full-page picture's page holds nothing else.
            self.alone.push(self.body.len());
        }
        self.body.push(p);
    }

    fn blocks(&mut self, s: &Section) {
        let look = self.look;
        let source = !s.generated;
        let mut after_break = true;
        let mut first = s.kind == Kind::Chapter;
        let mut lead = false;
        for b in &s.blocks {
            match b {
                Block::Para(spans) => {
                    let mut spans = spans.clone();
                    if (first && look.first_line == FirstLine::SmallCaps) || lead {
                        spans = self.lead_in(&spans);
                    }
                    let mut props = self.body_props(!after_break);
                    if first && let FirstLine::DropCap(lines) = look.first_line {
                        props.drop_cap = Some(lines);
                    }
                    self.body.push(self.para(&spans, props, &self.body_chars(), source));
                    after_break = false;
                    first = false;
                    lead = false;
                }
                Block::Break => {
                    let mut props = self.centered(0.0, 0.0);
                    props.align = Some(Align::Center);
                    props.line_spacing = Some(LineSpacing::Exactly(self.line() * 2.0));
                    props.keep_next = Some(true);
                    let mark = if look.break_space { String::new() } else { look.break_mark.clone() };
                    self.body.push(self.para(&[Span::plain(mark)], props, &self.body_chars(), false));
                    after_break = true;
                    lead = look.break_space;
                }
                Block::Verse(lines) => {
                    let mut spans: Vec<Span> = Vec::new();
                    for (i, line) in lines.iter().enumerate() {
                        if i > 0 {
                            spans.push(Span::plain(LINE_BREAK.to_string()));
                        }
                        spans.extend(line.iter().cloned());
                    }
                    let em = look.body_size;
                    let props = ParaProps {
                        align: Some(Align::Left),
                        indent_left: Some(3.0 * em),
                        indent_first: Some(-em),
                        line_spacing: Some(LineSpacing::Exactly(self.line())),
                        space_before: Some(self.line() * 0.5),
                        space_after: Some(self.line() * 0.5),
                        widow_control: Some(true),
                        ..ParaProps::default()
                    };
                    self.body.push(self.para(&spans, props, &self.body_chars(), source));
                    after_break = true;
                    first = false;
                }
                Block::Quote(paras) => {
                    let em = look.body_size;
                    for (i, p) in paras.iter().enumerate() {
                        let mut props = self.body_props(i > 0);
                        props.indent_left = Some(1.5 * em);
                        props.indent_right = Some(1.5 * em);
                        if i == 0 {
                            props.space_before = Some(self.line() * 0.5);
                        }
                        if i + 1 == paras.len() {
                            props.space_after = Some(self.line() * 0.5);
                        }
                        self.body.push(self.para(p, props, &self.body_chars(), source));
                    }
                    after_break = true;
                    first = false;
                }
                Block::Heading(spans) => {
                    let mut c = self.body_chars();
                    c.small_caps = Some(true);
                    let mut props = self.centered(self.line(), self.line() * 0.5);
                    props.align = Some(Align::Center);
                    self.body.push(self.para(spans, props, &c, source));
                    after_break = true;
                }
                Block::Centered(spans, scale) => {
                    let c = self.chars(&look.heading_font, look.body_size * f32::from(*scale) / 100.0);
                    let mut props = self.centered(0.0, self.line() * 0.6);
                    props.align = Some(Align::Center);
                    props.keep_next = None;
                    self.body.push(self.para(spans, props, &c, source));
                }
                Block::Entry(spans, id) => {
                    let mut props = self.body_props(false);
                    props.align = Some(Align::Left);
                    props.space_after = Some(self.line() * 0.25);
                    props.tabs = Some(vec![TabStop { pos: self.text_width, align: TabAlign::Right, leader: TabLeader::Dot }]);
                    let mut spans = spans.clone();
                    // Before the first layout the numbers are not known: a placeholder of a
                    // typical width keeps the contents the length it will be.
                    let folio = self.pass.folios.get(id).cloned().unwrap_or_else(|| "000".into());
                    spans.push(Span::plain(format!("\t{folio}")));
                    self.body.push(self.para(&spans, props, &self.body_chars(), false));
                }
                Block::Image { bytes, pixels, alt, fit } => {
                    self.image(bytes, *pixels, alt, *fit, source);
                    after_break = true;
                }
            }
        }
        if !s.notes.is_empty() {
            let mut c = self.body_chars();
            c.small_caps = Some(true);
            let mut props = self.centered(self.line() * 1.5, self.line() * 0.5);
            props.align = Some(Align::Center);
            self.body.push(self.para(&[Span::plain("Notes")], props, &c, false));
            let small = self.chars(&look.body_font, look.body_size * 0.88);
            for (i, n) in s.notes.iter().enumerate() {
                let mut spans = vec![Span { text: format!("{}.", i + 1), superscript: false, ..Span::default() }, Span::plain(" ")];
                spans.extend(n.iter().cloned());
                let mut props = self.body_props(false);
                props.line_spacing = Some(LineSpacing::Exactly(self.line() * 0.9));
                props.space_after = Some(self.line() * 0.2);
                let mut p = self.para(&spans, props, &small, source);
                // The note's number is the book's own.
                if let Some(r) = p.runs.first_mut() {
                    let _ = GENERATED.set(&mut r.props, &true);
                }
                self.body.push(p);
            }
        }
    }
}

struct Bands {
    blank_head: u32,
    blank_foot: u32,
    folio_foot: u32,
    heads: BTreeMap<(String, bool), u32>,
}

fn band(doc: &mut Document, kind: PartKind, p: Paragraph) -> u32 {
    doc.add_part(kind, vec![Arc::new(DocBlock::Para(p))])
}

fn head_paragraph(look: &Look, text: &str, recto: bool, width: f32, folio: bool) -> Paragraph {
    let mut c = CharProps { font: Some(look.body_font.clone()), size: Some(look.head_size), ..CharProps::default() };
    let mut p = Paragraph::new();
    p.mark = c.clone();
    let shown = if look.head_small_caps {
        c.small_caps = Some(true);
        c.spacing = Some(0.8);
        text.to_lowercase()
    } else {
        text.to_owned()
    };
    if folio {
        p.props.tabs = Some(vec![
            TabStop { pos: width / 2.0, align: TabAlign::Center, leader: TabLeader::None },
            TabStop { pos: width, align: TabAlign::Right, leader: TabLeader::None },
        ]);
        let plain = CharProps { font: Some(look.body_font.clone()), size: Some(look.head_size), ..CharProps::default() };
        if recto {
            let _ = p.insert_text(0, "\t", &plain);
            let at = p.text.len();
            let _ = p.insert_text(at, &shown, &c);
            let at = p.text.len();
            let _ = p.insert_text(at, "\t", &plain);
            let at = p.text.len();
            let _ = p.insert_object(at, page_field(), &plain);
        } else {
            let _ = p.insert_object(0, page_field(), &plain);
            let at = p.text.len();
            let _ = p.insert_text(at, "\t", &plain);
            let at = p.text.len();
            let _ = p.insert_text(at, &shown, &c);
        }
    } else {
        p.props.align = Some(Align::Center);
        let _ = p.insert_text(0, &shown, &c);
    }
    p
}

impl Bands {
    fn new(doc: &mut Document, look: &Look) -> Bands {
        let plain = CharProps { font: Some(look.body_font.clone()), size: Some(look.head_size), ..CharProps::default() };
        let mut empty = Paragraph::new();
        empty.mark = plain.clone();
        let blank_head = band(doc, PartKind::Header, empty.clone());
        let blank_foot = band(doc, PartKind::Footer, empty);
        let mut folio = Paragraph::new();
        folio.props.align = Some(Align::Center);
        folio.mark = plain.clone();
        let _ = folio.insert_object(0, page_field(), &plain);
        let folio_foot = band(doc, PartKind::Footer, folio);
        Bands { blank_head, blank_foot, folio_foot, heads: BTreeMap::new() }
    }

    fn head(&mut self, doc: &mut Document, look: &Look, text: &str, recto: bool, width: f32) -> u32 {
        if let Some(id) = self.heads.get(&(text.to_owned(), recto)) {
            return *id;
        }
        let id = band(doc, PartKind::Header, head_paragraph(look, text, recto, width, look.folio_head));
        self.heads.insert((text.to_owned(), recto), id);
        id
    }
}

/// The print document for `book` with `look` and the choices of this layout `pass`, and each
/// section's first body block (its page-map anchor).
pub fn print_document(book: &PrintBook, look: &Look, pass: &Pass) -> Result<(Document, Vec<(String, usize)>), Error> {
    let (tw, th) = look.trim;
    let (inside, outside, top, bottom) = look.margins;
    let inside = inside + pass.gutter;
    if !(tw > 1.0 && th > 1.0 && inside + outside < tw - 1.0 && top + bottom < th - 1.0) {
        return Err(Error::Setup(format!("a {tw} × {th} in page cannot hold these margins")));
    }
    if !(0.0..=0.5).contains(&look.bleed) {
        return Err(Error::Setup(format!("a bleed of {} in is not one a printer takes", look.bleed)));
    }
    // With bleed the page is larger on the outside, top and bottom; the margins are kept from
    // the trim.
    let b = look.bleed;
    let (w, h, outside, top, bottom) = (tw + b, th + 2.0 * b, outside + b, top + b, bottom + b);
    if look.body_font.is_empty() || look.heading_font.is_empty() {
        return Err(Error::Setup("the design names no font".into()));
    }
    let mut doc = jcj_doc::new_document();
    doc.settings.mirror_margins = true;
    doc.settings.even_odd_headers = true;
    doc.settings.auto_hyphenation = look.hyphenate;
    doc.settings.consecutive_hyphen_limit = if look.strict_hyphens { 1 } else { 2 };
    // A book's justified lines are hyphenated whenever a word would leave a wide gap, not only
    // past a word processor's quarter-inch zone.
    doc.settings.hyphenation_zone = look.body_size * 0.25;
    doc.core.title.clone_from(&book.title);
    doc.core.creator.clone_from(&book.author);
    doc.core.created = String::new();
    let text_width = (w - inside - outside) * INCH;
    let text_height = (h - top - bottom) * INCH;
    let mut bands = Bands::new(&mut doc, look);

    let mut b = Builder {
        look,
        lang: if book.lang.is_empty() { "en".into() } else { book.lang.clone() },
        pass,
        body: Vec::new(),
        media: BTreeMap::new(),
        alone: Vec::new(),
        text_width,
        text_height,
    };
    let mut starts = Vec::new();
    let mut props_of: Vec<SectionProps> = Vec::new();
    let mut front_sections = Vec::new();
    let first_body = book.sections.iter().position(|s| !s.front);
    // The section props each print section ends with, in order (blank pages are sections too).
    let mut ends: Vec<usize> = Vec::new();
    for (i, s) in book.sections.iter().enumerate() {
        if i > 0 && pass.blanks.contains(&i) {
            let mut empty = b.para(&[], b.body_props(false), &b.body_chars(), false);
            empty.props.page_break_before = Some(false);
            // The blank page is a section of its own with no head or folio.
            let mut sp = section_props(look, &mut bands, &mut doc, s, w, h, inside, outside, top, bottom, text_width, true);
            // It belongs to the section before it (a blank page before the first chapter is front matter).
            let front = i > 0 && book.sections[i - 1].front;
            sp.page_num_format = if front { NumFormat::LowerRoman } else { NumFormat::Decimal };
            ends.push(b.body.len());
            b.body.push(empty);
            if front {
                front_sections.push(props_of.len());
            }
            props_of.push(sp);
        }
        starts.push((s.id.clone(), b.body.len()));
        if !matches!(s.kind, Kind::Plain) || s.label.is_some() || !s.title.is_empty() {
            b.opening(s);
        } else {
            // A plain page (title page, copyright) starts a little way down.
            let mut p = b.centered(b.sink(0.15), 0.0);
            p.line_spacing = Some(LineSpacing::Exactly(1.0));
            b.body.push(b.para(&[], p, &b.body_chars(), false));
        }
        b.blocks(s);
        let mut sp = section_props(look, &mut bands, &mut doc, s, w, h, inside, outside, top, bottom, text_width, false);
        sp.page_num_format = if s.front { NumFormat::LowerRoman } else { NumFormat::Decimal };
        if i == 0 || Some(i) == first_body {
            sp.page_num_start = Some(1);
        }
        if s.front {
            front_sections.push(props_of.len());
        }
        ends.push(b.body.len().saturating_sub(1));
        props_of.push(sp);
    }
    // The model keeps a section's settings on its last paragraph; the last section's are the
    // document's.
    let last = props_of
        .pop()
        .unwrap_or_else(|| section_props(look, &mut bands, &mut doc, &Section::default(), w, h, inside, outside, top, bottom, text_width, true));
    for (sp, end) in props_of.into_iter().zip(ends.iter()) {
        if let Some(p) = b.body.get_mut(*end) {
            p.section = Some(Box::new(sp));
        }
    }
    doc.last_section = last;
    let _ = FRONT.set(&mut doc, &front_sections);
    for i in std::mem::take(&mut b.alone) {
        if let Some(next) = b.body.get_mut(i + 1) {
            next.props.page_break_before = Some(true);
        }
    }
    doc.media = b.media;
    doc.body = b.body.into_iter().map(|p| Arc::new(DocBlock::Para(p))).collect();
    if doc.body.is_empty() {
        doc.body.push(Arc::new(DocBlock::Para(Paragraph::new())));
    }
    Ok((doc, starts))
}

#[allow(clippy::too_many_arguments, reason = "the page's geometry, passed through once")]
fn section_props(
    look: &Look,
    bands: &mut Bands,
    doc: &mut Document,
    s: &Section,
    w: f32,
    h: f32,
    inside: f32,
    outside: f32,
    top: f32,
    bottom: f32,
    text_width: f32,
    blank: bool,
) -> SectionProps {
    let mut sp = SectionProps {
        page_w: w * INCH,
        page_h: h * INCH,
        margin_left: inside * INCH,
        margin_right: outside * INCH,
        margin_top: top * INCH,
        margin_bottom: bottom * INCH,
        header: ((top - look.bleed) * INCH * 0.45).max(18.0) + look.bleed * INCH,
        footer: ((bottom - look.bleed) * INCH * 0.45).max(18.0) + look.bleed * INCH,
        title_page: true,
        ..SectionProps::default()
    };
    sp.start = SectionStart::NextPage;
    let shows_heads = !blank && look.heads != Heads::None && matches!(s.kind, Kind::Chapter) && !s.front;
    let (verso, recto) = match look.heads {
        Heads::AuthorTitle => (s_author(doc), doc.core.title.clone()),
        Heads::TitleChapter => (doc.core.title.clone(), if s.head.is_empty() { doc.core.title.clone() } else { s.head.clone() }),
        Heads::None => (String::new(), String::new()),
    };
    let folio_pages = look.folios && !blank && !matches!(s.kind, Kind::Plain);
    if shows_heads {
        sp.headers.even = Some(bands.head(doc, look, &verso, false, text_width));
        sp.headers.default = Some(bands.head(doc, look, &recto, true, text_width));
    } else if look.folio_head && folio_pages {
        sp.headers.even = Some(bands.head(doc, look, "", false, text_width));
        sp.headers.default = Some(bands.head(doc, look, "", true, text_width));
    } else {
        sp.headers.even = Some(bands.blank_head);
        sp.headers.default = Some(bands.blank_head);
    }
    sp.headers.first = Some(bands.blank_head);
    let foot = if folio_pages && !look.folio_head { bands.folio_foot } else { bands.blank_foot };
    sp.footers.default = Some(foot);
    sp.footers.even = Some(foot);
    // An opening page shows its folio at the foot (a drop folio), unless it has none at all.
    sp.footers.first = Some(if folio_pages && matches!(s.kind, Kind::Chapter) { bands.folio_foot } else { bands.blank_foot });
    sp
}

fn s_author(doc: &Document) -> String {
    doc.core.creator.clone()
}
