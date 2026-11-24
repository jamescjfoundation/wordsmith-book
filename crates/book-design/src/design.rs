// SPDX-License-Identifier: MIT OR Apache-2.0
//! A design (DSN-1, plan section 6.5, `docs/design-format.md`): a small TOML file with a schema
//! version. Body and heading fonts by role, sizes, leading, indents, justification and
//! hyphenation, widows and orphans; the chapter opening; scene breaks; the part page; running
//! heads and folios; and the ebook's choices. A design is data, never code: print and the ebook
//! read its values.

use serde::{Deserialize, Serialize};

/// The schema version this build writes and the newest it reads.
pub const SCHEMA: u32 = 1;
/// The largest design file.
pub const MAX_FILE: usize = 64 << 10;

fn is_default<T: Default + PartialEq>(v: &T) -> bool {
    *v == T::default()
}

/// How hard hyphenation tries.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Hyphenation {
    /// No hyphenation.
    Off,
    /// The usual: at most two hyphenated lines in a row.
    #[default]
    Normal,
    /// Fewer hyphens: never two hyphenated lines in a row.
    Strict,
}

/// The first line of a chapter.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FirstLine {
    /// Nothing special.
    None,
    /// The first words in small capitals.
    #[default]
    SmallCaps,
    /// A drop cap.
    DropCap,
}

/// Where a chapter starts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Start {
    /// On the next right-hand page.
    #[default]
    Recto,
    /// On the next page.
    Any,
}

/// How a line of the opening is set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Case {
    /// As written.
    #[default]
    AsWritten,
    /// In small capitals.
    SmallCaps,
    /// In capitals.
    Capitals,
}

/// How text lines up.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Align {
    /// Centered.
    #[default]
    Center,
    /// At the left.
    Left,
}

/// A scene break.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BreakStyle {
    /// A mark (a glyph or ornament) centered on its own line.
    #[default]
    Mark,
    /// A blank line, then the next scene starting in small capitals (a mark is shown where the
    /// break falls at a page's head or foot).
    Space,
}

/// Running heads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Heads {
    /// The author on left-hand pages, the title on right-hand pages.
    #[default]
    AuthorTitle,
    /// The title on left-hand pages, the chapter on right-hand pages.
    TitleChapter,
    /// No running heads.
    None,
}

/// Where folios (page numbers) go.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Folio {
    /// Centered at the foot.
    #[default]
    Foot,
    /// At the outer corner of the head, beside the running head.
    Head,
}

/// Body text.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Body {
    /// The font, by role.
    pub font: String,
    /// The size, points.
    pub size: f32,
    /// Line height, a multiple of the size.
    pub leading: f32,
    /// First-line indent, ems.
    pub indent: f32,
    /// Justified (else ragged right).
    pub justify: bool,
    /// Hyphenation.
    pub hyphenation: Hyphenation,
    /// Fewest lines of a paragraph left at the foot of a page.
    pub orphans: u8,
    /// Fewest lines of a paragraph carried to the head of a page.
    pub widows: u8,
}

impl Default for Body {
    fn default() -> Self {
        Body {
            font: "classic-serif".into(),
            size: 11.0,
            leading: 1.3,
            indent: 1.2,
            justify: true,
            hyphenation: Hyphenation::Normal,
            orphans: 2,
            widows: 2,
        }
    }
}

/// The chapter opening.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Chapter {
    /// The heading font, by role.
    pub font: String,
    /// Where chapters start.
    pub start: Start,
    /// How far down the page the opening starts, a fraction of the text block (0 to 0.5).
    pub sink: f32,
    /// The number line's size.
    pub number_size: f32,
    /// The number line's case.
    pub number_case: Case,
    /// The title's size.
    pub title_size: f32,
    /// The title's case.
    pub title_case: Case,
    /// How the opening lines up.
    pub align: Align,
    /// An ornament under the title (a glyph, or empty for none).
    pub ornament: String,
    /// The first line of the text.
    pub first_line: FirstLine,
    /// A drop cap's height in lines (2 to 4).
    pub drop_lines: u8,
}

impl Default for Chapter {
    fn default() -> Self {
        Chapter {
            font: "classic-serif".into(),
            start: Start::Recto,
            sink: 0.25,
            number_size: 11.0,
            number_case: Case::SmallCaps,
            title_size: 20.0,
            title_case: Case::AsWritten,
            align: Align::Center,
            ornament: String::new(),
            first_line: FirstLine::SmallCaps,
            drop_lines: 3,
        }
    }
}

/// Scene breaks.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SceneBreak {
    /// A mark or a blank line.
    pub style: BreakStyle,
    /// The mark ("* * *", "⁂", a fleuron).
    pub mark: String,
}

impl Default for SceneBreak {
    fn default() -> Self {
        SceneBreak { style: BreakStyle::Mark, mark: "* * *".into() }
    }
}

/// The part page.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Part {
    /// The number line's size.
    pub number_size: f32,
    /// The title's size.
    pub title_size: f32,
    /// How far down the page, a fraction of the text block.
    pub sink: f32,
}

impl Default for Part {
    fn default() -> Self {
        Part { number_size: 14.0, title_size: 24.0, sink: 0.33 }
    }
}

/// Running heads and folios.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RunningHeads {
    /// What the heads show.
    pub style: Heads,
    /// Where the folio goes.
    pub folio: Folio,
    /// Their size.
    pub size: f32,
    /// In small capitals.
    pub small_caps: bool,
}

impl Default for RunningHeads {
    fn default() -> Self {
        RunningHeads { style: Heads::AuthorTitle, folio: Folio::Foot, size: 9.0, small_caps: true }
    }
}

/// The ebook's choices: the same design as conservative CSS.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Ebook {
    /// Justified (else ragged right).
    pub justify: bool,
    /// First-line indent, ems.
    pub indent: f32,
    /// The first line of a chapter (a drop cap becomes small capitals, which every reading
    /// system shows).
    pub first_line: FirstLine,
    /// Headings in the heading font when fonts are embedded.
    pub heading_font: bool,
}

impl Default for Ebook {
    fn default() -> Self {
        Ebook { justify: true, indent: 1.2, first_line: FirstLine::SmallCaps, heading_font: true }
    }
}

/// A design.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Design {
    /// [`SCHEMA`].
    pub schema: u32,
    /// The design's id: letters, digits, `-` and `_`.
    pub id: String,
    /// Its name as shown.
    pub name: String,
    /// A line about it.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub about: String,
    /// The genres it suits (the chooser filters by them).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub genres: Vec<String>,
    /// Its moods.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub moods: Vec<String>,
    /// Body text.
    #[serde(default, skip_serializing_if = "is_default")]
    pub body: Body,
    /// The chapter opening.
    #[serde(default, skip_serializing_if = "is_default")]
    pub chapter: Chapter,
    /// Scene breaks.
    #[serde(default, skip_serializing_if = "is_default")]
    pub scene_break: SceneBreak,
    /// The part page.
    #[serde(default, skip_serializing_if = "is_default")]
    pub part: Part,
    /// Running heads and folios.
    #[serde(default, skip_serializing_if = "is_default")]
    pub heads: RunningHeads,
    /// The ebook.
    #[serde(default, skip_serializing_if = "is_default")]
    pub ebook: Ebook,
}

impl Default for Design {
    fn default() -> Self {
        Design {
            schema: SCHEMA,
            id: "untitled".into(),
            name: "Untitled".into(),
            about: String::new(),
            genres: Vec::new(),
            moods: Vec::new(),
            body: Body::default(),
            chapter: Chapter::default(),
            scene_break: SceneBreak::default(),
            part: Part::default(),
            heads: RunningHeads::default(),
            ebook: Ebook::default(),
        }
    }
}

/// Why a design file could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The file is larger than [`MAX_FILE`].
    TooLarge,
    /// The file is from a newer version of the app.
    Newer(u32),
    /// The file is not a design.
    NotADesign(String),
    /// The design's values are not usable: every problem, in plain words.
    Invalid(Vec<String>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::TooLarge => write!(f, "the design file is larger than {} KB", MAX_FILE >> 10),
            Error::Newer(v) => {
                write!(f, "this design was made by a newer version of Wordsmith Book (design format {v}; this version reads up to {SCHEMA})")
            }
            Error::NotADesign(why) => write!(f, "this is not a design file: {why}"),
            Error::Invalid(problems) => write!(f, "the design cannot be used: {}", problems.join(" ")),
        }
    }
}

impl std::error::Error for Error {}

fn range(out: &mut Vec<String>, what: &str, v: f32, lo: f32, hi: f32, unit: &str) {
    if !(lo..=hi).contains(&v) || v.is_nan() {
        out.push(format!("The {what} must be between {lo} and {hi}{unit}; it is {v}."));
    }
}

/// Every problem with a design, in plain words (none: it can be used). `known_role` says whether
/// a font role is one Book has.
pub fn problems(d: &Design, known_role: &dyn Fn(&str) -> bool) -> Vec<String> {
    let mut out = Vec::new();
    if d.id.is_empty() || d.id.len() > 100 || !d.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        out.push(format!("The design's id must be letters, digits, “-” and “_”; it is “{}”.", d.id));
    }
    if d.name.trim().is_empty() {
        out.push("The design needs a name.".into());
    }
    for (what, role) in [("body font", &d.body.font), ("chapter font", &d.chapter.font)] {
        if !known_role(role) {
            out.push(format!("The {what} “{role}” is not one of Book's font roles."));
        }
    }
    range(&mut out, "body size", d.body.size, 7.0, 24.0, " points");
    range(&mut out, "leading", d.body.leading, 1.0, 2.0, " times the size");
    range(&mut out, "first-line indent", d.body.indent, 0.0, 4.0, " ems");
    for (what, v) in [("orphans", d.body.orphans), ("widows", d.body.widows)] {
        if !(1..=4).contains(&v) {
            out.push(format!("The fewest {what} must be between 1 and 4 lines; it is {v}."));
        }
    }
    range(&mut out, "chapter sink", d.chapter.sink, 0.0, 0.5, " of the page");
    range(&mut out, "chapter number size", d.chapter.number_size, 7.0, 72.0, " points");
    range(&mut out, "chapter title size", d.chapter.title_size, 7.0, 72.0, " points");
    if d.chapter.first_line == FirstLine::DropCap && !(2..=4).contains(&d.chapter.drop_lines) {
        out.push(format!("A drop cap must be 2 to 4 lines high; it is {}.", d.chapter.drop_lines));
    }
    if d.chapter.ornament.chars().count() > 8 {
        out.push("The chapter ornament must be a few characters at most.".into());
    }
    if d.scene_break.style == BreakStyle::Mark && d.scene_break.mark.trim().is_empty() {
        out.push("A scene break shown as a mark needs a mark.".into());
    }
    if d.scene_break.mark.chars().count() > 12 || d.scene_break.mark.chars().any(char::is_alphanumeric) {
        out.push("A scene break's mark is a few symbols, with no letters or digits.".into());
    }
    range(&mut out, "part number size", d.part.number_size, 7.0, 72.0, " points");
    range(&mut out, "part title size", d.part.title_size, 7.0, 72.0, " points");
    range(&mut out, "part sink", d.part.sink, 0.0, 0.5, " of the page");
    range(&mut out, "running head size", d.heads.size, 6.0, 14.0, " points");
    range(&mut out, "ebook indent", d.ebook.indent, 0.0, 4.0, " ems");
    out
}

/// Turn a version 0 design into version 1. Version 0 was never released: it exists only so the
/// migration path is built and tested (body text was in a `[text]` table).
fn migrate_0(t: &mut toml::Table) {
    if let Some(text) = t.remove("text") {
        t.insert("body".into(), text);
    }
    t.insert("schema".into(), toml::Value::Integer(1));
}

/// Read a design file, migrating an older one, then check its values with `known_role`.
pub fn read(text: &str, known_role: &dyn Fn(&str) -> bool) -> Result<Design, Error> {
    if text.len() > MAX_FILE {
        return Err(Error::TooLarge);
    }
    let mut t: toml::Table = text.parse().map_err(|e: toml::de::Error| Error::NotADesign(e.message().to_owned()))?;
    let schema = match t.get("schema") {
        Some(toml::Value::Integer(n)) if *n >= 0 => u32::try_from(*n).map_err(|_| Error::NotADesign("the schema is not a version".into()))?,
        _ => return Err(Error::NotADesign("it has no schema version".into())),
    };
    if schema > SCHEMA {
        return Err(Error::Newer(schema));
    }
    if schema == 0 {
        migrate_0(&mut t);
    }
    let d: Design = t.try_into().map_err(|e: toml::de::Error| Error::NotADesign(e.message().to_owned()))?;
    let problems = problems(&d, known_role);
    if problems.is_empty() { Ok(d) } else { Err(Error::Invalid(problems)) }
}

/// Write a design file; the same design always gives the same text.
pub fn write(d: &Design) -> Result<String, Error> {
    let mut d = d.clone();
    d.schema = SCHEMA;
    toml::to_string(&d).map(|s| format!("# A Wordsmith Book design (docs/design-format.md).\n{s}")).map_err(|e| Error::NotADesign(e.to_string()))
}
