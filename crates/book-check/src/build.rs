// SPDX-License-Identifier: MIT OR Apache-2.0
//! Building an edition's print files: the book view (with the fixes that wait on the writer), the
//! assembled book, its design and the edition's profile become a laid-out book.

use std::collections::BTreeMap;

use wordsmith_book_design::design::Design;
use wordsmith_book_design::mix;
use wordsmith_book_format::assemble::{Assembled, assemble};
use wordsmith_book_model::{BookProject, Edition};
use wordsmith_book_typeset::from_book::{Left, print_book};
use wordsmith_book_typeset::{Look, PrintBook, Typeset, typeset};
use wordsmith_manuscript_format::book::Book;

use crate::profile::{self, Profile};

/// A built print edition.
pub struct Print {
    /// The assembled book.
    pub assembled: Assembled,
    /// What was printed.
    pub book: PrintBook,
    /// How.
    pub look: Look,
    /// The layout.
    pub typeset: Typeset,
    /// The edition's profile.
    pub profile: Profile,
    /// What was left out.
    pub left: Left,
}

/// Build `edition` of the project from `book` (the source's view, with waiting fixes applied),
/// the project's own designs (`designs`, by id), and `images` (a picture's bytes by its path).
pub fn print(
    project: &BookProject,
    designs: &BTreeMap<String, String>,
    book: &Book,
    edition: &Edition,
    images: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Result<Print, String> {
    if !edition.format.is_print() {
        return Err(format!("the {} is not a printed edition", edition.name.to_lowercase()));
    }
    let profile_id = edition.profile_id();
    let profile = profile::get(profile_id).ok_or_else(|| format!("there is no print profile “{profile_id}”"))?;
    let geometry = profile.geometry(edition)?;
    let design = print_design(project, designs, edition)?;
    let assembled = assemble(project, book, Some(edition));
    let lang = if book.metadata.info.language.is_empty() { "en".to_owned() } else { book.metadata.info.language.clone() };
    let (pbook, look, left) = print_book(&assembled, &design, &geometry, &lang, images).map_err(|e| e.to_string())?;
    let typeset = typeset(&pbook, &look, if project.date.is_empty() { "2000-01-01" } else { &project.date }).map_err(|e| e.to_string())?;
    Ok(Print { assembled, book: pbook, look, typeset, profile, left })
}

/// The design a print edition is set in: the book's, or its large-print variant for a large-print
/// edition (PRT-11).
pub fn print_design(project: &BookProject, designs: &BTreeMap<String, String>, edition: &Edition) -> Result<Design, String> {
    let design = mix::resolve(&project.design, designs).map_err(|e| e.to_string())?;
    Ok(if edition.format == wordsmith_book_model::Format::LargePrint { wordsmith_book_design::large_print::variant(&design) } else { design })
}

/// A proof of print `edition` (PRT-13): the edition as built, set again with "PROOF" across
/// every page and its lines numbered in the margin. The pages break as the edition's do (the
/// numbers sit in the margin); it never declares the PDF standards. Export never writes one for
/// an edition; only a proof asked for is.
pub fn proof(
    project: &BookProject,
    designs: &BTreeMap<String, String>,
    book: &Book,
    edition: &Edition,
    images: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Result<Print, String> {
    let mut pr = print(project, designs, book, edition, images)?;
    pr.look.proof = true;
    pr.look.pdf_standards = false;
    pr.typeset = typeset(&pr.book, &pr.look, if project.date.is_empty() { "2000-01-01" } else { &project.date }).map_err(|e| e.to_string())?;
    Ok(pr)
}

/// Build only the opening of print `edition`: the front matter and the first chapter (or part
/// and chapter), so a preview can show its first pages at once while the whole book is built.
/// The contents' page numbers are placeholders until the whole book is laid out.
pub fn print_opening(
    project: &BookProject,
    designs: &BTreeMap<String, String>,
    book: &Book,
    edition: &Edition,
    images: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Result<Print, String> {
    if !edition.format.is_print() {
        return Err(format!("the {} is not a printed edition", edition.name.to_lowercase()));
    }
    let profile_id = edition.profile_id();
    let profile = profile::get(profile_id).ok_or_else(|| format!("there is no print profile “{profile_id}”"))?;
    let geometry = profile.geometry(edition)?;
    let design = print_design(project, designs, edition)?;
    let assembled = assemble(project, book, Some(edition));
    let lang = if book.metadata.info.language.is_empty() { "en".to_owned() } else { book.metadata.info.language.clone() };
    let (mut pbook, look, left) = print_book(&assembled, &design, &geometry, &lang, images).map_err(|e| e.to_string())?;
    let first = pbook.sections.iter().position(|s| s.kind == wordsmith_book_typeset::Kind::Chapter).map_or(pbook.sections.len(), |i| i + 1);
    pbook.sections.truncate(first);
    let typeset = typeset(&pbook, &look, if project.date.is_empty() { "2000-01-01" } else { &project.date }).map_err(|e| e.to_string())?;
    Ok(Print { assembled, book: pbook, look, typeset, profile, left })
}

fn is_contents_piece(p: &wordsmith_book_format::assemble::Piece) -> bool {
    matches!(
        p.what,
        wordsmith_book_format::assemble::What::Element(wordsmith_book_model::ElementKind::Contents)
            | wordsmith_book_format::assemble::What::Matter(wordsmith_manuscript_format::book::FrontKind::Contents)
    )
}

/// Build ebook `edition` of the project from `book`, the project's own designs and `images`, as
/// [`print`] does for print.
pub fn ebook(
    project: &BookProject,
    designs: &BTreeMap<String, String>,
    book: &Book,
    edition: &Edition,
    images: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Result<(Assembled, wordsmith_book_epub::Ebook), String> {
    if edition.format.is_print() {
        return Err(format!("the {} is not an ebook", edition.name.to_lowercase()));
    }
    let design = mix::resolve(&project.design, designs).map_err(|e| e.to_string())?;
    let assembled = assemble(project, book, Some(edition));
    let settings = wordsmith_book_epub::Settings {
        language: book.metadata.info.language.clone(),
        date: if project.date.is_empty() { "2000-01-01".to_owned() } else { project.date.clone() },
        seed: project.date.clone(),
        decorative: project.decorative.clone(),
        page_marks: Vec::new(),
        page_source: String::new(),
    };
    let mut settings = settings;
    // The page list: where the first print edition's pages start, in the ebook's terms.
    if edition.page_list
        && let Some(paper) = project.editions.iter().find(|e| e.format.is_print())
        && let Ok(pr) = print(project, designs, book, paper, images)
    {
        let shown: Vec<String> =
            assembled.pieces().filter(|p| wordsmith_book_epub::in_ebook(p) && !is_contents_piece(p)).map(|p| p.id.clone()).collect();
        let order: Vec<String> = pr.assembled.pieces().map(|p| p.id.clone()).collect();
        let starts = pr.typeset.page_starts();
        for (i, s) in starts.iter().enumerate() {
            let mut mark = wordsmith_book_epub::PageMark {
                number: s.number,
                label: s.label.clone(),
                piece: s.section.clone(),
                unit: s.unit,
                offset: s.offset,
                at_end: false,
            };
            // A page inside its section with none of the author's text at its top (a blank page, a
            // picture) is marked at the next of the section's paragraphs to start a page, or after
            // the section's text.
            let first_of_section = i == 0 || starts[i - 1].section != s.section;
            if s.unit.is_none() && !first_of_section {
                match starts[i + 1..].iter().take_while(|n| n.section == s.section).find_map(|n| n.unit) {
                    Some(u) => mark.unit = Some(u),
                    None => mark.at_end = true,
                }
            }
            // A page of a piece the ebook does not show (the half title, the contents, a print-only
            // chapter) is marked where the next piece it shows starts.
            if !shown.contains(&s.section) {
                mark.at_end = false;
                let from = order.iter().position(|id| *id == s.section).unwrap_or(0);
                mark.piece = order[from..].iter().find(|id| shown.contains(id)).or(shown.last()).cloned().unwrap_or_default();
                mark.unit = None;
                mark.offset = 0;
            }
            settings.page_marks.push(mark);
        }
        settings.page_source = if paper.isbn.is_empty() {
            format!("the {} edition", paper.name.to_lowercase())
        } else {
            format!("urn:isbn:{}", paper.isbn.replace('-', ""))
        };
    }
    let ebook = wordsmith_book_epub::ebook(&assembled, &design, edition, &settings, images).map_err(|e| e.to_string())?;
    Ok((assembled, ebook))
}
