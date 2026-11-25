// SPDX-License-Identifier: MIT OR Apache-2.0
//! Print profiles (PRE-4, plan section 9.1): trims, bleed, the gutter by page count, margin
//! minimums, paper thickness per page, page-count limits, blank-page runs, the smallest type, the
//! spine-text threshold and the cover's barcode area, as data in `assets/print/`. Each records
//! when it was last checked and against what kind of source.

use serde::Deserialize;
use wordsmith_book_model::{Edition, Paper};
use wordsmith_book_typeset::from_book::Geometry;

/// One trim a profile offers.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trim {
    /// Its id (`6x9`).
    pub id: String,
    /// Its name as shown.
    pub name: String,
    /// Width, inches.
    pub width: f32,
    /// Height, inches.
    pub height: f32,
    /// Default margins: inside (before the gutter), outside, top, bottom, inches.
    pub margins: [f32; 4],
}

/// A step of the gutter table.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gutter {
    /// Books of up to this many pages.
    pub pages: usize,
    /// Get this gutter, inches.
    pub inches: f32,
}

/// Per paper: a value for white, cream and color.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerPaper<T> {
    /// White paper.
    pub white: T,
    /// Cream paper.
    pub cream: T,
    /// Color paper.
    pub color: T,
}

impl<T: Copy> PerPaper<T> {
    /// The value for `paper`.
    pub fn of(&self, paper: Paper) -> T {
        match paper {
            Paper::White => self.white,
            Paper::Cream => self.cream,
            Paper::Color => self.color,
        }
    }
}

/// Margin minimums, inches.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Minimum {
    /// Outside margin without bleed.
    pub outside: f32,
    /// Outside margin with bleed.
    pub outside_bleed: f32,
    /// Top margin.
    pub top: f32,
    /// Bottom margin.
    pub bottom: f32,
}

/// A print profile.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    /// Its id.
    pub id: String,
    /// Its name as shown.
    pub name: String,
    /// When its rules were last checked.
    pub checked: String,
    /// Against what kind of source.
    pub source: String,
    /// Bleed, inches.
    pub bleed: f32,
    /// The smallest type, points.
    pub min_type: f32,
    /// The most blank pages in a row inside the book.
    pub blank_run_inside: usize,
    /// The most blank pages at the end.
    pub blank_run_end: usize,
    /// Spine text only on books of at least this many pages.
    pub spine_text_min_pages: usize,
    /// The barcode area on the back cover, inches (width, height).
    pub barcode: [f32; 2],
    /// The fewest pages.
    pub min_pages: usize,
    /// The page count must be even.
    pub even_pages: bool,
    /// Paper thickness per page, inches.
    pub paper: PerPaper<f64>,
    /// The most pages, by paper.
    pub max_pages: PerPaper<usize>,
    /// Margin minimums.
    pub minimum: Minimum,
    /// Trims.
    pub trims: Vec<Trim>,
    /// The gutter by page count.
    pub gutter: Vec<Gutter>,
}

/// The built-in profiles, by id.
pub const FILES: [(&str, &str); 3] = [
    ("pod-paperback", include_str!("../../../assets/print/pod-paperback.toml")),
    ("pod-hardcover", include_str!("../../../assets/print/pod-hardcover.toml")),
    ("generic-pdf", include_str!("../../../assets/print/generic-pdf.toml")),
];

/// Read a profile file.
pub fn read(text: &str) -> Result<Profile, String> {
    let p: Profile = toml::from_str(text).map_err(|e| e.message().to_owned())?;
    if p.trims.is_empty() || p.gutter.is_empty() {
        return Err("a profile needs trims and a gutter table".into());
    }
    if p.gutter.windows(2).any(|w| w[0].pages >= w[1].pages) {
        return Err("the gutter table must grow by page count".into());
    }
    Ok(p)
}

/// The built-in profile `id`.
pub fn get(id: &str) -> Option<Profile> {
    FILES.iter().find(|(i, _)| *i == id).and_then(|(_, t)| read(t).ok())
}

impl Profile {
    /// The trim `id`.
    pub fn trim(&self, id: &str) -> Option<&Trim> {
        self.trims.iter().find(|t| t.id == id)
    }

    /// The gutter for a book of `pages` pages.
    pub fn gutter_for(&self, pages: usize) -> f32 {
        self.gutter.iter().find(|g| pages <= g.pages).or(self.gutter.last()).map_or(0.0, |g| g.inches)
    }

    /// The page geometry for an edition: its trim's size and margins, and this profile's gutter
    /// table. Unknown trims fall back to the profile's first.
    pub fn geometry(&self, edition: &Edition) -> Result<Geometry, String> {
        let t = self.trim(&edition.trim).ok_or_else(|| format!("the {} profile has no “{}” trim", self.name.to_lowercase(), edition.trim))?;
        Ok(Geometry {
            trim: (t.width, t.height),
            margins: (t.margins[0], t.margins[1], t.margins[2], t.margins[3]),
            gutter: self.gutter.iter().map(|g| (g.pages, g.inches)).collect(),
            bleed: if edition.bleed { self.bleed } else { 0.0 },
            notes: match edition.notes {
                wordsmith_book_model::NoteStyle::Popup => wordsmith_book_typeset::NotePlace::Footnotes,
                wordsmith_book_model::NoteStyle::ChapterEnd => wordsmith_book_typeset::NotePlace::ChapterEnd,
                wordsmith_book_model::NoteStyle::BookEnd => wordsmith_book_typeset::NotePlace::BookEnd,
            },
            min_size: if edition.format == wordsmith_book_model::Format::LargePrint {
                wordsmith_book_design::large_print::LARGE_PRINT_SIZE
            } else {
                0.0
            },
        })
    }

    /// The spine width for `pages` pages of `paper`, inches.
    pub fn spine(&self, pages: usize, paper: Paper) -> f64 {
        pages as f64 * self.paper.of(paper)
    }
}
