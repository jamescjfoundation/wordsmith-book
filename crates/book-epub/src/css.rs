// SPDX-License-Identifier: MIT OR Apache-2.0
//! The ebook's stylesheet from a design's ebook choices (EBK-1, SPEC 6.8): conservative CSS in
//! `em`, no fixed body size, no positioning and no floats but the drop cap, so a reading
//! system's own settings keep working.

use wordsmith_book_design::design::{Align, BreakStyle, Case, Design, FirstLine};

fn case(c: Case) -> &'static str {
    match c {
        Case::AsWritten => "",
        Case::SmallCaps => " font-variant: small-caps; letter-spacing: 0.05em;",
        Case::Capitals => " text-transform: uppercase; letter-spacing: 0.08em;",
    }
}

/// The stylesheet. `fonts` is the font-face rules and the body and heading families when the
/// edition embeds fonts (M6.2); empty otherwise.
pub fn stylesheet(d: &Design, fonts: &Fonts) -> String {
    let e = &d.ebook;
    let align = if e.justify { "justify" } else { "left" };
    let heading_align = match d.chapter.align {
        Align::Center => "center",
        Align::Left => "left",
    };
    let mut s = String::from("/* Made from the book's design; sizes in em so reading settings apply. */\n");
    s += &fonts.faces;
    let body_family = fonts.body.as_deref().map(|f| format!(" font-family: {f};")).unwrap_or_default();
    let heading_family =
        if e.heading_font { fonts.heading.as_deref().map(|f| format!(" font-family: {f};")).unwrap_or_default() } else { String::new() };
    s += &format!("body {{ margin: 0 5%;{body_family} }}\n");
    s += &format!("p {{ margin: 0; text-indent: {:.2}em; text-align: {align}; hyphens: auto; -webkit-hyphens: auto; }}\n", e.indent);
    s += "h1 + p, h2 + p, h3 + p, h4 + p, header + p, hr + p, blockquote + p, .verse + p, figure + p, p.first, p.centered, .opening p:first-of-type { text-indent: 0; }\n";
    s += &format!(
        "header {{ text-align: {heading_align}; margin: 3em 0 2em 0;{heading_family} }}\nheader .label {{ margin: 0; text-indent: 0; text-align: {heading_align}; font-size: 0.9em;{} }}\n",
        case(d.chapter.number_case)
    );
    s += &format!("h1 {{ font-size: 1.6em; font-weight: normal; margin: 0.3em 0 0 0;{} }}\n", case(d.chapter.title_case));
    s += "header .subtitle { margin: 0.5em 0 0 0; text-indent: 0; text-align: inherit; font-style: italic; }\n";
    s += "h2 { font-size: 1.25em; font-weight: normal; margin: 1.5em 0 0.5em 0; }\nh3, h4 { font-size: 1.1em; font-weight: normal; font-style: italic; margin: 1.2em 0 0.4em 0; }\n";
    s += ".part header { margin-top: 30%; }\n.part h1 { font-size: 2em; }\n";
    s += ".epigraph { margin: 0 10% 2em 10%; font-style: italic; }\n.epigraph p { text-indent: 0; text-align: left; }\n.epigraph .attribution { text-align: right; font-style: normal; }\n";
    s += "p.centered { text-align: center; }\n.matter-plain { margin-top: 20%; text-align: center; }\n.matter-plain p { text-indent: 0; text-align: center; margin: 0.5em 0; }\n";
    s += "blockquote { margin: 1em 2em; }\nblockquote p { text-indent: 0; }\n.verse { margin: 1em 2em; }\n.verse p { text-indent: -1em; padding-left: 1em; text-align: left; }\n";
    s += match d.scene_break.style {
        BreakStyle::Mark => {
            "hr.break { border: none; margin: 1em 0; height: 1.5em; text-align: center; }\nhr.break::after { content: attr(data-mark); }\np.break { text-align: center; text-indent: 0; margin: 0.75em 0; }\n"
        }
        _ => "hr.break { border: none; margin: 0; height: 1.5em; }\np.break { text-indent: 0; margin: 0; height: 1.5em; }\n",
    };
    s += match e.first_line {
        FirstLine::None => "",
        FirstLine::SmallCaps => ".lead { font-variant: small-caps; letter-spacing: 0.05em; }\n",
        FirstLine::DropCap => {
            ".drop { float: left; font-size: 3.2em; line-height: 0.85; margin: 0.05em 0.08em 0 0; }\n.lead { font-variant: small-caps; }\n"
        }
    };
    s += ".sc { font-variant: small-caps; }\n.strike { text-decoration: line-through; }\n";
    s += "figure { margin: 1em 0; text-align: center; }\nfigure img { max-width: 100%; height: auto; }\nfigure.page img { max-height: 95vh; }\n";
    s += "a.noteref { vertical-align: super; font-size: 0.75em; line-height: 0; text-decoration: none; }\nsection.notes { margin-top: 2em; font-size: 0.9em; }\nsection.notes h2 { font-size: 1.1em; }\nsection.notes li p { text-indent: 0; }\n";
    s += "nav ol { list-style: none; padding-left: 0; }\nnav ol ol { padding-left: 1.5em; }\nnav li { margin: 0.3em 0; }\n";
    s
}

/// Embedded fonts, when the edition embeds them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Fonts {
    /// `@font-face` rules.
    pub faces: String,
    /// The body family list.
    pub body: Option<String>,
    /// The heading family list.
    pub heading: Option<String>,
}
