// SPDX-License-Identifier: MIT OR Apache-2.0
//! The ebook preview's layout (EBK-8, plan 5.8): a chapter in the design's ebook mapping (the
//! same choices the ebook's CSS carries) laid out through the core at a device's screen size,
//! with a text size, no running heads and no folios. It is a preview: reading apps lay out
//! ebooks their own way.

use wordsmith_book_design::Design;
use wordsmith_book_design::design::FirstLine as DesignFirstLine;
use wordsmith_book_format::assemble::Assembled;

use crate::from_book::{Geometry, look, print_book};
use crate::{Error, FirstLine, Heads, Kind, Look, PrintBook, Typeset, typeset};

/// A screen the preview shows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Device {
    /// Its id.
    pub id: &'static str,
    /// Its name as shown.
    pub name: &'static str,
    /// The text area's width and height, inches.
    pub size: (f32, f32),
    /// The margin around the text, inches.
    pub margin: f32,
}

/// The screens: a phone, an e-ink reader and a tablet.
pub const DEVICES: [Device; 3] = [
    Device { id: "phone", name: "Phone", size: (2.6, 4.9), margin: 0.15 },
    Device { id: "e-ink", name: "E-ink reader", size: (3.6, 4.8), margin: 0.25 },
    Device { id: "tablet", name: "Tablet", size: (5.8, 7.9), margin: 0.45 },
];

/// The text sizes, as (name, body size in points).
pub const TEXT_SIZES: [(&str, f32); 4] = [("Small", 9.0), ("Medium", 11.0), ("Large", 13.5), ("Extra large", 16.5)];

/// A device by id.
pub fn device(id: &str) -> Option<Device> {
    DEVICES.into_iter().find(|d| d.id == id)
}

/// The look of a design's ebook mapping on a device at a text size (points).
pub fn ebook_look(design: &Design, device: &Device, text: f32) -> Result<Look, Error> {
    let m = device.margin;
    let geometry = Geometry { trim: device.size, margins: (m, m, m, m * 1.2), gutter: Vec::new(), bleed: 0.0 };
    let mut l = look(design, &geometry)?;
    let e = &design.ebook;
    let scale = text / l.body_size.max(1.0);
    l.body_size = text;
    l.leading = l.leading.max(1.3);
    l.indent = e.indent;
    l.justify = e.justify;
    l.first_line = match e.first_line {
        DesignFirstLine::None => FirstLine::Plain,
        DesignFirstLine::SmallCaps => FirstLine::SmallCaps,
        DesignFirstLine::DropCap => FirstLine::DropCap(design.chapter.drop_lines.clamp(2, 3)),
    };
    if !e.heading_font {
        l.heading_font.clone_from(&l.body_font);
    }
    // Headings keep their size relative to the text, as the CSS's em sizes do.
    l.title_size = (l.title_size * scale).min(text * 1.8);
    l.number_size *= scale;
    l.sink = 0.06;
    l.heads = Heads::None;
    l.folios = false;
    Ok(l)
}

/// The book's first chapter (or its first section, when it has no chapters) laid out on a device
/// at a text size. `images` gives a picture's bytes by its path.
pub fn first_chapter(
    a: &Assembled,
    design: &Design,
    device: &Device,
    text: f32,
    lang: &str,
    images: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Result<Typeset, Error> {
    let look = ebook_look(design, device, text)?;
    let (book, _, _) = print_book(a, design, &Geometry { trim: device.size, ..Geometry::default() }, lang, images)?;
    let section = book
        .sections
        .iter()
        .find(|s| s.kind == Kind::Chapter)
        .or_else(|| book.sections.iter().find(|s| !s.blocks.is_empty()))
        .cloned()
        .ok_or_else(|| Error::Setup("the book has no chapter to preview".into()))?;
    let mut section = section;
    section.recto = false;
    let one = PrintBook { title: book.title.clone(), author: book.author.clone(), lang: book.lang.clone(), sections: vec![section] };
    typeset(&one, &look, "2000-01-01")
}
