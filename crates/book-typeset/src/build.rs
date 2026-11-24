// SPDX-License-Identifier: MIT OR Apache-2.0
//! A [`PrintBook`] and a [`Look`] to a print document in the core's model.

use std::sync::Arc;

use jcj_doc::para::InlineObject;
use jcj_doc::props::{Align, CharProps, LineSpacing, ParaProps};
use jcj_doc::section::SectionStart;
use jcj_doc::{Block as DocBlock, Document, Paragraph, PartKind, SectionProps};

use crate::{Block, Error, FirstLine, INCH, Look, PrintBook, Span};

/// The line break inside a paragraph (verse).
const LINE_BREAK: char = '\n';

struct Builder<'a> {
    look: &'a Look,
    body: Vec<Paragraph>,
}

impl Builder<'_> {
    fn chars(&self, font: &str, size: f32) -> CharProps {
        CharProps { font: Some(font.to_owned()), size: Some(size), ..CharProps::default() }
    }

    fn body_chars(&self) -> CharProps {
        self.chars(&self.look.body_font, self.look.body_size)
    }

    fn line(&self) -> f32 {
        self.look.body_size * self.look.leading
    }

    fn body_props(&self, indent: bool) -> ParaProps {
        ParaProps {
            align: Some(Align::Justify),
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
            align: Some(Align::Center),
            indent_first: Some(0.0),
            space_before: Some(before),
            space_after: Some(after),
            keep_next: Some(true),
            line_spacing: Some(LineSpacing::Multiple(1.0)),
            ..ParaProps::default()
        }
    }

    fn para(&self, spans: &[Span], props: ParaProps, base: &CharProps) -> Paragraph {
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
            let at = p.text.len();
            // Text with characters the model cannot hold is a bug in the caller; it never panics.
            let _ = p.insert_text(at, &s.text, &c);
        }
        p
    }

    fn lead_in(&self, spans: &[Span]) -> Vec<Span> {
        let mut out = spans.to_vec();
        if self.look.first_line != FirstLine::SmallCaps {
            return out;
        }
        if let Some(first) = out.first_mut() {
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
        }
        out
    }

    fn opening(&mut self, s: &crate::Section) {
        let heading = self.chars(&self.look.heading_font, self.look.number_size);
        let mut sunk = false;
        if let Some(n) = &s.number {
            let mut c = heading.clone();
            c.small_caps = Some(true);
            c.spacing = Some(1.5);
            self.body.push(self.para(&[Span::plain(n)], self.centered(self.look.sink * INCH, self.line()), &c));
            sunk = true;
        }
        if !s.title.is_empty() {
            let c = self.chars(&self.look.heading_font, self.look.title_size);
            let before = if sunk { 0.0 } else { self.look.sink * INCH };
            self.body.push(self.para(&[Span::plain(&s.title)], self.centered(before, self.line()), &c));
            sunk = true;
        }
        if !self.look.ornament.is_empty() {
            let c = self.chars(&self.look.body_font, self.look.title_size * 0.8);
            self.body.push(self.para(&[Span::plain(&self.look.ornament)], self.centered(0.0, self.line()), &c));
        }
        if sunk {
            self.body.push(self.para(&[], self.centered(0.0, self.line()), &self.body_chars()));
        }
    }

    fn blocks(&mut self, blocks: &[Block]) {
        let mut after_break = true;
        let mut first = true;
        for b in blocks {
            match b {
                Block::Para(spans) => {
                    let spans = if first { self.lead_in(spans) } else { spans.clone() };
                    let mut props = self.body_props(!after_break);
                    if first && let FirstLine::DropCap(lines) = self.look.first_line {
                        props.drop_cap = Some(lines);
                    }
                    self.body.push(self.para(&spans, props, &self.body_chars()));
                    after_break = false;
                    first = false;
                }
                Block::Break => {
                    let mark = if self.look.break_mark.is_empty() { String::new() } else { self.look.break_mark.clone() };
                    let mut props = self.centered(0.0, 0.0);
                    props.line_spacing = Some(LineSpacing::Exactly(self.line() * 2.0));
                    self.body.push(self.para(&[Span::plain(mark)], props, &self.body_chars()));
                    after_break = true;
                }
                Block::Verse(lines) => {
                    let mut spans: Vec<Span> = Vec::new();
                    for (i, line) in lines.iter().enumerate() {
                        if i > 0 {
                            spans.push(Span::plain(LINE_BREAK.to_string()));
                        }
                        spans.extend(line.iter().cloned());
                    }
                    let em = self.look.body_size;
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
                    self.body.push(self.para(&spans, props, &self.body_chars()));
                    after_break = true;
                    first = false;
                }
                Block::Quote(paras) => {
                    let em = self.look.body_size;
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
                        self.body.push(self.para(p, props, &self.body_chars()));
                    }
                    after_break = true;
                    first = false;
                }
                Block::Heading(spans) => {
                    let mut c = self.body_chars();
                    c.small_caps = Some(true);
                    self.body.push(self.para(spans, self.centered(self.line(), self.line() * 0.5), &c));
                    after_break = true;
                }
            }
        }
    }
}

fn band(doc: &mut Document, kind: PartKind, text: Option<&str>, field: bool, c: &CharProps) -> u32 {
    let mut p = Paragraph::new();
    p.props.align = Some(Align::Center);
    p.mark = c.clone();
    if let Some(t) = text {
        let mut sc = c.clone();
        sc.small_caps = Some(true);
        sc.spacing = Some(0.8);
        let _ = p.insert_text(0, &t.to_lowercase(), &sc);
    }
    if field {
        let obj = InlineObject::Field { instr: "PAGE".into(), result: String::new(), locked: false, dirty: false, code: false, form: String::new() };
        let at = p.text.len();
        let _ = p.insert_object(at, obj, c);
    }
    doc.add_part(kind, vec![Arc::new(DocBlock::Para(p))])
}

/// The print document for `book` with `look`, and each section's first body block (its page-map
/// anchor). A blank page goes before each section whose index is in `blanks` (so it opens on a
/// right-hand page); the caller finds them by laying out ([`crate::typeset`]).
pub fn print_document(book: &PrintBook, look: &Look, blanks: &std::collections::BTreeSet<usize>) -> Result<(Document, Vec<(String, usize)>), Error> {
    let (w, h) = look.trim;
    let (inside, outside, top, bottom) = look.margins;
    if !(w > 1.0 && h > 1.0 && inside + outside < w - 1.0 && top + bottom < h - 1.0) {
        return Err(Error::Setup(format!("a {w} × {h} in page cannot hold these margins")));
    }
    if look.body_font.is_empty() || look.heading_font.is_empty() {
        return Err(Error::Setup("the design names no font".into()));
    }
    let mut doc = jcj_doc::new_document();
    doc.settings.mirror_margins = true;
    doc.settings.even_odd_headers = true;
    doc.settings.auto_hyphenation = look.hyphenate;
    doc.settings.consecutive_hyphen_limit = 2;
    doc.core.title.clone_from(&book.title);
    doc.core.creator.clone_from(&book.author);
    doc.core.created = String::new();

    let head = CharProps { font: Some(look.body_font.clone()), size: Some(look.head_size), ..CharProps::default() };
    let odd = band(&mut doc, PartKind::Header, look.heads.then_some(book.title.as_str()), false, &head);
    let even = band(&mut doc, PartKind::Header, look.heads.then_some(book.author.as_str()), false, &head);
    let blank_head = band(&mut doc, PartKind::Header, None, false, &head);
    let folio = band(&mut doc, PartKind::Footer, None, true, &head);
    let none = band(&mut doc, PartKind::Footer, None, false, &head);

    // Every section starts on the next page; right-hand starts come from blank pages Book adds
    // (the model's "odd page" is by page number, and a book's is by the leaf).
    let section = |first: bool, blank: bool| -> SectionProps {
        let mut s = SectionProps {
            page_w: w * INCH,
            page_h: h * INCH,
            margin_left: inside * INCH,
            margin_right: outside * INCH,
            margin_top: top * INCH,
            margin_bottom: bottom * INCH,
            header: (top * INCH * 0.45).max(18.0),
            footer: (bottom * INCH * 0.45).max(18.0),
            title_page: true,
            ..SectionProps::default()
        };
        s.start = SectionStart::NextPage;
        s.headers.default = Some(if blank { blank_head } else { odd });
        s.headers.even = Some(if blank { blank_head } else { even });
        s.headers.first = Some(blank_head);
        s.footers.default = Some(if blank { none } else { folio });
        s.footers.even = Some(if blank { none } else { folio });
        s.footers.first = Some(none);
        if first {
            s.page_num_start = Some(1);
        }
        s
    };

    let mut b = Builder { look, body: Vec::new() };
    let mut starts = Vec::new();
    for (i, s) in book.sections.iter().enumerate() {
        if i > 0 {
            // The section that ends here is the previous one: the model keeps a section's settings
            // on its last paragraph.
            if let Some(last) = b.body.last_mut() {
                last.section = Some(Box::new(section(i == 1, false)));
            }
            if blanks.contains(&i) {
                let mut empty = b.para(&[], b.body_props(false), &b.body_chars());
                empty.section = Some(Box::new(section(false, true)));
                b.body.push(empty);
            }
        }
        starts.push((s.id.clone(), b.body.len()));
        b.opening(s);
        b.blocks(&s.blocks);
        if b.body.len() == starts.last().map_or(0, |(_, at)| *at) {
            b.body.push(b.para(&[], b.body_props(false), &b.body_chars()));
        }
    }
    doc.last_section = section(book.sections.len() <= 1, false);
    doc.body = b.body.into_iter().map(|p| Arc::new(DocBlock::Para(p))).collect();
    if doc.body.is_empty() {
        doc.body.push(Arc::new(DocBlock::Para(Paragraph::new())));
    }
    Ok((doc, starts))
}
