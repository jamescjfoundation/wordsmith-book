// SPDX-License-Identifier: MIT OR Apache-2.0
//! The Book layer: everything Wordsmith Book adds to a manuscript, kept apart from the words.
//! Editions, the design in use and its overrides, Book's own front and back matter, settings per
//! section, metadata additions, cleanup decisions and light fixes, all keyed by the source's
//! stable section ids (plan section 6.1). No files and no interface: `book-format` reads and
//! writes it, the views show it.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod document;
pub mod isbn;
pub mod matter;
pub mod numbers;
pub mod picture;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The stable id of a section of the book: the id Manuscript gives each item, which Book keys its
/// own layer by (plan section 6.1).
pub use wordsmith_manuscript_model::ItemId as SectionId;

/// How much a finding matters (preflight, metadata needs).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    /// Export is blocked until it is fixed.
    Blocks,
    /// Worth fixing; export goes ahead.
    Warning,
    /// For information.
    Note,
}

impl Severity {
    /// The severity's name as shown.
    pub fn title(self) -> &'static str {
        match self {
            Severity::Blocks => tl("Blocks export"),
            Severity::Warning => tl("Warning"),
            Severity::Note => tl("Note"),
        }
    }
}

/// The schema version of `book.toml` this build writes and the newest it reads. Until Book's
/// first release tag, version 1 is extended in place (plan section 6.1).
pub const SCHEMA: u32 = 1;

/// An interface text that the interface translates where it shows it (Book's interface catalogs
/// list it; see `book-ui`'s languages). The text itself.
pub const fn tl(english: &'static str) -> &'static str {
    english
}

fn is_default<T: Default + PartialEq>(v: &T) -> bool {
    *v == T::default()
}

fn is_false(b: &bool) -> bool {
    !*b
}

fn is_true(b: &bool) -> bool {
    *b
}

fn yes() -> bool {
    true
}

/// Where the book's words come from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Source {
    /// A Manuscript project (folder or `.wsmz`) Book reads and never writes: a path from the Book
    /// project's folder.
    Manuscript(String),
    /// A Manuscript-format folder inside the Book project that Book owns (an import).
    Owned(String),
}

impl Default for Source {
    fn default() -> Self {
        Source::Owned(OWNED_SOURCE.into())
    }
}

/// The folder name of an owned source inside the project.
pub const OWNED_SOURCE: &str = "source.wsm";

/// A contributor's role.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Role {
    /// Wrote the book.
    #[default]
    Author,
    /// Edited it.
    Editor,
    /// Translated it.
    Translator,
    /// Made its pictures.
    Illustrator,
    /// Wrote its foreword.
    Foreword,
    /// Wrote its introduction.
    Introduction,
    /// Designed its cover.
    CoverDesigner,
    /// Read it for an audio edition.
    Narrator,
}

impl Role {
    /// Every role, in menu order.
    pub const ALL: [Role; 8] =
        [Role::Author, Role::Editor, Role::Translator, Role::Illustrator, Role::Foreword, Role::Introduction, Role::CoverDesigner, Role::Narrator];

    /// The role's name as shown.
    pub fn title(self) -> &'static str {
        match self {
            Role::Author => "Author",
            Role::Editor => "Editor",
            Role::Translator => "Translator",
            Role::Illustrator => "Illustrator",
            Role::Foreword => "Foreword by",
            Role::Introduction => "Introduction by",
            Role::CoverDesigner => "Cover designer",
            Role::Narrator => "Narrator",
        }
    }
}

/// A person or organisation who contributed to the book.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contributor {
    /// The name as printed.
    pub name: String,
    /// What they did.
    #[serde(default)]
    pub role: Role,
}

/// The publisher's imprint as the title and copyright pages show it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Imprint {
    /// The imprint's name.
    pub name: String,
    /// Address lines.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub address: Vec<String>,
    /// Its website.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub website: String,
    /// Its logo, a path inside the project's `assets/`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub logo: String,
}

/// What Book adds to the source's metadata. Never written to a linked source.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata {
    /// Contributors besides the source's author, with their roles.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contributors: Vec<Contributor>,
    /// Subject codes (from a published subject scheme).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subject_codes: Vec<String>,
    /// The imprint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub imprint: Option<Imprint>,
    /// The copyright holder, when not the byline.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub copyright_holder: String,
    /// Values that replace the source's (title, subtitle, description, ...), by field name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub overrides: BTreeMap<String, String>,
    /// The author's other books, for "also by" (title, then series if any).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub also_by: Vec<OtherBook>,
    /// A short biography for "about the author".
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub about_author: String,
}

/// One of the author's other books.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtherBook {
    /// Its title.
    pub title: String,
    /// Its series, if any.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub series: String,
}

/// The parts of a design that can come from another design (DSN-3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DesignPart {
    /// Body text: fonts, size, leading, indents, hyphenation.
    Body,
    /// Chapter openings.
    ChapterOpening,
    /// Scene breaks.
    SceneBreaks,
    /// Running heads and folios.
    RunningHeads,
}

impl DesignPart {
    /// Every part.
    pub const ALL: [DesignPart; 4] = [DesignPart::Body, DesignPart::ChapterOpening, DesignPart::SceneBreaks, DesignPart::RunningHeads];
}

/// The design a book uses.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignChoice {
    /// The design's id: a built-in design or one of the project's own (`designs/<id>.toml`).
    pub id: String,
    /// Parts taken from other designs (DSN-3), by part.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub mix: BTreeMap<DesignPart, String>,
}

impl Default for DesignChoice {
    fn default() -> Self {
        DesignChoice { id: "classic".into(), mix: BTreeMap::new() }
    }
}

/// What an edition is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Format {
    /// A paperback.
    #[default]
    Paperback,
    /// A hardcover.
    Hardcover,
    /// An ebook.
    Ebook,
    /// A large-print paperback.
    LargePrint,
}

impl Format {
    /// Whether the edition is printed.
    pub fn is_print(self) -> bool {
        !matches!(self, Format::Ebook)
    }

    /// The format's name as shown.
    pub fn title(self) -> &'static str {
        match self {
            Format::Paperback => tl("Paperback"),
            Format::Hardcover => tl("Hardcover"),
            Format::Ebook => tl("Ebook"),
            Format::LargePrint => tl("Large print"),
        }
    }
    /// The print profile a printed edition of this format uses unless it names one.
    pub fn default_profile(self) -> &'static str {
        match self {
            Format::Hardcover => "pod-hardcover",
            Format::Paperback | Format::LargePrint => "pod-paperback",
            Format::Ebook => "",
        }
    }
}

/// Paper for a printed edition.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Paper {
    /// White paper.
    White,
    /// Cream paper.
    #[default]
    Cream,
    /// Color paper.
    Color,
}

impl Paper {
    /// The paper's key in print profiles.
    pub fn key(self) -> &'static str {
        match self {
            Paper::White => "white",
            Paper::Cream => "cream",
            Paper::Color => "color",
        }
    }
}

/// How an ebook edition shows notes (TXT-3, EBK-7).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NoteStyle {
    /// Pop-up footnotes where reading systems support them.
    #[default]
    Popup,
    /// Endnotes at the end of each chapter.
    ChapterEnd,
    /// Endnotes at the end of the book.
    BookEnd,
}

/// One edition of the book: its format, ISBN, and the settings that differ from the book's.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edition {
    /// The edition's id (`paperback`, `ebook`, ...), unique in the project; also the build
    /// record's file name.
    pub id: String,
    /// Its name as shown.
    pub name: String,
    /// Its format.
    pub format: Format,
    /// Its ISBN, if it has one.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub isbn: String,
    /// For print: the trim's id (`6x9`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub trim: String,
    /// For print: the paper.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paper: Option<Paper>,
    /// For print: the print profile's id (`pod-paperback`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub profile: String,
    /// Design values that differ for this edition, by key (`body.size = "11.5"`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub overrides: BTreeMap<String, String>,
    /// Price notes, free text.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub price_notes: String,
    /// For ebooks: embed the fonts (EBK-4).
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub embed_fonts: bool,
    /// For ebooks: how notes show.
    #[serde(default, skip_serializing_if = "is_default")]
    pub notes: NoteStyle,
    /// For store editions (STR-10): the store's name as the user gave it.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub store: String,
    /// For print: the pages bleed (PRT-7): printing runs past the cut on the outside, top and
    /// bottom by the profile's bleed, and full-page pictures fill the page to the edge.
    #[serde(default, skip_serializing_if = "is_false")]
    pub bleed: bool,
    /// For ebooks: a page list giving the print edition's page numbers (EBK-6).
    #[serde(default, skip_serializing_if = "is_false")]
    pub page_list: bool,
    /// For ebooks: the cover image, by its path in the project (`assets/cover.jpg`) (EBK-5).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub cover: String,
}

impl Edition {
    /// The default paperback: 6 × 9 in, cream paper, the print-on-demand paperback profile.
    pub fn paperback() -> Edition {
        Edition {
            id: "paperback".into(),
            name: "Paperback".into(),
            format: Format::Paperback,
            trim: "6x9".into(),
            paper: Some(Paper::Cream),
            profile: "pod-paperback".into(),
            embed_fonts: true,
            ..Edition::default()
        }
    }

    /// The default hardcover: case laminate, 6 × 9 in, cream paper, the print-on-demand hardcover
    /// profile.
    pub fn hardcover() -> Edition {
        Edition {
            id: "hardcover".into(),
            name: "Hardcover".into(),
            format: Format::Hardcover,
            profile: "pod-hardcover".into(),
            ..Edition::paperback()
        }
    }

    /// The print profile this edition uses: its own, else its format's.
    pub fn profile_id(&self) -> &str {
        if self.profile.is_empty() { self.format.default_profile() } else { &self.profile }
    }

    /// The default ebook.
    pub fn ebook() -> Edition {
        Edition { id: "ebook".into(), name: "Ebook".into(), format: Format::Ebook, embed_fonts: true, page_list: true, ..Edition::default() }
    }
}

/// Where an element of Book's own goes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Placement {
    /// Before the body.
    #[default]
    Front,
    /// After the body.
    Back,
}

/// A front or back matter element Book makes (STR-4 to STR-6, STR-8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ElementKind {
    /// The half title.
    HalfTitle,
    /// The title page.
    TitlePage,
    /// The copyright page.
    Copyright,
    /// A dedication.
    Dedication,
    /// An epigraph.
    Epigraph,
    /// The contents.
    Contents,
    /// About the author.
    AboutTheAuthor,
    /// The author's other books.
    AlsoBy,
    /// Acknowledgements.
    Acknowledgements,
    /// A foreword.
    Foreword,
    /// A preface.
    Preface,
    /// An introduction.
    Introduction,
    /// A prologue.
    Prologue,
    /// An epilogue.
    Epilogue,
    /// An afterword.
    Afterword,
    /// A newsletter or website page.
    Newsletter,
    /// An excerpt from the next book.
    Excerpt,
    /// A glossary.
    Glossary,
    /// The book's endnotes.
    Endnotes,
    /// Discussion questions.
    Questions,
    /// Anything else.
    Other,
}

impl ElementKind {
    /// The element's name as shown and its default title.
    pub fn title(self) -> &'static str {
        match self {
            ElementKind::HalfTitle => tl("Half title"),
            ElementKind::TitlePage => tl("Title page"),
            ElementKind::Copyright => tl("Copyright"),
            ElementKind::Dedication => tl("Dedication"),
            ElementKind::Epigraph => tl("Epigraph"),
            ElementKind::Contents => tl("Contents"),
            ElementKind::AboutTheAuthor => tl("About the Author"),
            ElementKind::AlsoBy => tl("Also By"),
            ElementKind::Acknowledgements => tl("Acknowledgements"),
            ElementKind::Foreword => tl("Foreword"),
            ElementKind::Preface => tl("Preface"),
            ElementKind::Introduction => tl("Introduction"),
            ElementKind::Prologue => tl("Prologue"),
            ElementKind::Epilogue => tl("Epilogue"),
            ElementKind::Afterword => tl("Afterword"),
            ElementKind::Newsletter => tl("Stay in Touch"),
            ElementKind::Excerpt => tl("An Excerpt"),
            ElementKind::Glossary => tl("Glossary"),
            ElementKind::Endnotes => tl("Notes"),
            ElementKind::Questions => tl("Questions for Discussion"),
            ElementKind::Other => tl("Untitled"),
        }
    }

    /// Where an element of this kind sits among the others on its side of the book, by
    /// convention (lower first): the title pages, dedication and epigraph, contents, then the
    /// foreword, preface and introduction, with a prologue last before the text; after the text,
    /// an epilogue and afterword, then notes, glossary and questions, an excerpt, and the pages
    /// about the author.
    pub fn rank(self, placement: Placement) -> u8 {
        match (self, placement) {
            (ElementKind::HalfTitle, _) => 0,
            (ElementKind::TitlePage, _) => 1,
            (ElementKind::Copyright, _) => 2,
            (ElementKind::Dedication, _) => 3,
            (ElementKind::Epigraph, _) => 4,
            (ElementKind::Contents, _) => 5,
            (ElementKind::Foreword, _) => 6,
            (ElementKind::Preface, _) => 7,
            (ElementKind::Acknowledgements, Placement::Front) => 8,
            (ElementKind::Introduction, _) => 9,
            (ElementKind::Prologue, _) => 10,
            (ElementKind::Epilogue, _) => 20,
            (ElementKind::Afterword, _) => 21,
            (ElementKind::Acknowledgements, Placement::Back) => 22,
            (ElementKind::Endnotes, _) => 23,
            (ElementKind::Glossary, _) => 24,
            (ElementKind::Questions, _) => 25,
            (ElementKind::Excerpt, _) => 26,
            (ElementKind::AboutTheAuthor, _) => 27,
            (ElementKind::AlsoBy, _) => 28,
            (ElementKind::Newsletter, _) => 29,
            (ElementKind::Other, _) => 30,
        }
    }

    /// Where the element goes by default.
    pub fn placement(self) -> Placement {
        match self {
            ElementKind::AboutTheAuthor
            | ElementKind::AlsoBy
            | ElementKind::Acknowledgements
            | ElementKind::Epilogue
            | ElementKind::Afterword
            | ElementKind::Newsletter
            | ElementKind::Excerpt
            | ElementKind::Glossary
            | ElementKind::Endnotes
            | ElementKind::Questions => Placement::Back,
            _ => Placement::Front,
        }
    }
}

/// A paragraph of a section shown in one kind of output only: found by the words it starts with
/// (unique within the section, its scenes included).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockOnly {
    /// The paragraph's first words, exactly as written.
    pub starts: String,
    /// Where it shows.
    pub only: Only,
}

impl BlockOnly {
    /// Where a block whose plain text is `text` shows, by these rules (the first that matches).
    pub fn of(rules: &[BlockOnly], text: &str) -> Only {
        let t = text.trim_start();
        rules.iter().find(|r| !r.starts.is_empty() && t.starts_with(r.starts.as_str())).map_or(Only::Both, |r| r.only)
    }
}

/// Which outputs a piece of the book appears in (STR-9).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Only {
    /// Print and ebook.
    #[default]
    Both,
    /// Print editions only.
    Print,
    /// Ebook editions only.
    Ebook,
}

/// An element of Book's own: generated from metadata where it can be, and editable.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Element {
    /// Its id, unique in the project.
    pub id: String,
    /// What it is.
    pub kind: Option<ElementKind>,
    /// Front or back.
    #[serde(default)]
    pub placement: Placement,
    /// Its title (empty: the kind's title).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub title: String,
    /// Its text as a template (`{title}`, `{byline}`, ...); empty: generated from metadata.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub text: String,
    /// Which outputs show it.
    #[serde(default, skip_serializing_if = "is_default")]
    pub only: Only,
    /// Store editions that show it (empty: all) (STR-10).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stores: Vec<String>,
}

/// Book's settings for one section of the source.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionSettings {
    /// A subtitle under the chapter's title.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub subtitle: String,
    /// An epigraph: its text.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub epigraph: String,
    /// The epigraph's attribution.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub epigraph_source: String,
    /// Which outputs show the section.
    #[serde(default, skip_serializing_if = "is_default")]
    pub only: Only,
    /// Paragraphs shown in one kind of output only (STR-9), by the words each starts with.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocks_only: Vec<BlockOnly>,
    /// Hide the chapter's number.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub unnumbered: bool,
}

/// How chapters are numbered (STR-2).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NumberStyle {
    /// "Chapter One".
    #[default]
    LabelWords,
    /// "Chapter 1".
    LabelFigures,
    /// "One".
    Words,
    /// "1".
    Figures,
    /// "I".
    Roman,
    /// No number.
    None,
}

/// Chapter and part numbering.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Numbering {
    /// Chapters' number style.
    #[serde(default, skip_serializing_if = "is_default")]
    pub chapters: NumberStyle,
    /// Parts' number style.
    #[serde(default, skip_serializing_if = "is_default")]
    pub parts: NumberStyle,
    /// The word before a chapter number ("Chapter").
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub chapter_label: String,
    /// The word before a part number ("Part").
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub part_label: String,
}

/// A light fix (STR-7, plan section 6.4): exact words in one paragraph of a section become others.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fix {
    /// The fix's id, unique in the project; for a linked source, also the offer's id.
    pub id: String,
    /// The section whose text it changes.
    pub section: Option<SectionId>,
    /// The exact words (plain text, within one paragraph, once in the section).
    pub find: String,
    /// What they become.
    pub replace: String,
    /// Why.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub note: String,
    /// When it was made (`YYYY-MM-DD` or RFC 3339).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub made: String,
    /// For a linked source: the offer was written to the offers file.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub offered: bool,
}

/// The import rules the user set in the import review (IMP-4), kept for re-imports.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportRules {
    /// Rules switched off, by name.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub off: Vec<String>,
    /// Extra chapter title patterns (plain words a chapter line starts with).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chapter_words: Vec<String>,
    /// Extra scene-break marks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub break_marks: Vec<String>,
}

/// Cleanup decisions (IMP-6).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cleanup {
    /// "Keep exactly as written": no cleanup proposed.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub keep_exactly: bool,
    /// Kinds of change switched off (`quotes`, `dashes`, `ellipses`, `spaces`, `breaks`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub off: Vec<String>,
    /// The import's rules.
    #[serde(default, skip_serializing_if = "is_default")]
    pub rules: ImportRules,
}

/// A Book project's own layer (`book.toml`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookProject {
    /// The schema version ([`SCHEMA`]).
    pub schema: u32,
    /// Where the words come from.
    pub source: Source,
    /// The project's date (`YYYY-MM-DD`): what builds print as their date, so a build can be
    /// repeated exactly.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub date: String,
    /// The design in use.
    #[serde(default)]
    pub design: DesignChoice,
    /// Chapter and part numbering.
    #[serde(default, skip_serializing_if = "is_default")]
    pub numbering: Numbering,
    /// Metadata additions.
    #[serde(default, skip_serializing_if = "is_default")]
    pub metadata: Metadata,
    /// Editions, in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub editions: Vec<Edition>,
    /// Book's own front and back matter, in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<Element>,
    /// Settings per section of the source.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sections: BTreeMap<SectionId, SectionSettings>,
    /// Cleanup decisions and import rules.
    #[serde(default, skip_serializing_if = "is_default")]
    pub cleanup: Cleanup,
    /// Light fixes, oldest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixes: Vec<Fix>,
    /// Pictures marked decorative, by their path in the source: they need no alternative text
    /// and are hidden from assistive technology (SPEC 6.6).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decorative: Vec<String>,
    /// No-break text and hyphenation hints (TXT-5).
    #[serde(default, skip_serializing_if = "is_default")]
    pub hints: Hints,
}

/// Most entries kept in each list of hints.
pub const MAX_HINTS: usize = 2000;
/// The longest hint, in characters.
pub const MAX_HINT_LEN: usize = 200;

/// How lines may break inside some words (TXT-5), for print and ebook alike.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hints {
    /// Words and phrases never broken across lines ("Mr. Smith", "10 kg"): their spaces become
    /// no-break spaces and their words are not hyphenated.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keep: Vec<String>,
    /// Words with the places they may break marked by hyphens ("man-u-script"); a word written
    /// without one is never hyphenated.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hyphenate: Vec<String>,
}

impl Hints {
    /// What is wrong with the hints, one line each.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (what, list) in [("kept together", &self.keep), ("hyphenation", &self.hyphenate)] {
            if list.len() > MAX_HINTS {
                out.push(format!("There are more than {MAX_HINTS} {what} hints."));
            }
            for e in list {
                if e.trim().is_empty() {
                    out.push(format!("A {what} hint is empty."));
                } else if e.chars().count() > MAX_HINT_LEN {
                    out.push(format!("A {what} hint is longer than {MAX_HINT_LEN} characters."));
                }
            }
        }
        for e in &self.hyphenate {
            let t = e.trim();
            if t.chars().any(char::is_whitespace) {
                out.push(format!("“{t}” is more than one word; a hyphenation hint is one word."));
            } else if t.starts_with('-') || t.ends_with('-') || t.contains("--") {
                out.push(format!("“{t}” has a hyphen at an end or two together; mark each place a word may break with one hyphen."));
            } else if !t.is_empty() && !t.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '\'' || c == '\u{2019}') {
                out.push(format!("“{t}” has a character that is not a letter or digit."));
            }
        }
        for e in &self.keep {
            if !e.trim().is_empty() && !e.trim().chars().any(char::is_whitespace) && !e.chars().any(char::is_alphanumeric) {
                out.push(format!("“{}” has no word to keep together.", e.trim()));
            }
        }
        out
    }
}

impl BookProject {
    /// A new project on `source`, with a paperback and an ebook edition.
    pub fn new(source: Source, date: &str) -> BookProject {
        BookProject {
            schema: SCHEMA,
            source,
            date: date.to_owned(),
            editions: vec![Edition::paperback(), Edition::ebook()],
            ..BookProject::default()
        }
    }

    /// The edition with `id`.
    pub fn edition(&self, id: &str) -> Option<&Edition> {
        self.editions.iter().find(|e| e.id == id)
    }

    /// Book's settings for a section (defaults when none are set).
    pub fn section(&self, id: SectionId) -> SectionSettings {
        self.sections.get(&id).cloned().unwrap_or_default()
    }

    /// Problems that make the project unusable: duplicate ids, an empty id or edition name.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for e in &self.editions {
            if e.id.trim().is_empty() || !seen.insert(e.id.as_str()) {
                out.push(format!("edition ids must be present and unique (`{}`)", e.id));
            }
            if !e.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                out.push(format!("edition id `{}` may hold only letters, digits and hyphens", e.id));
            }
        }
        let mut seen = std::collections::BTreeSet::new();
        for e in &self.elements {
            if e.id.trim().is_empty() || !seen.insert(e.id.as_str()) {
                out.push(format!("element ids must be present and unique (`{}`)", e.id));
            }
        }
        let mut seen = std::collections::BTreeSet::new();
        for f in &self.fixes {
            if f.id.trim().is_empty() || !seen.insert(f.id.as_str()) {
                out.push(format!("fix ids must be present and unique (`{}`)", f.id));
            }
            if f.find.is_empty() || f.find.contains('\n') || f.replace.contains('\n') {
                out.push(format!("fix `{}` must change words within one paragraph", f.id));
            }
        }
        out
    }
}
