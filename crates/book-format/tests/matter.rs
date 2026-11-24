// SPDX-License-Identifier: MIT OR Apache-2.0
//! Front and back matter (plan M3.1): each element made from the fixture's metadata, edits that
//! round-trip through the format, and a source's own pages used instead of Book's, never doubled.

use std::path::Path;

use wordsmith_book_format::assemble::{What, assemble};
use wordsmith_book_format::toml_file;
use wordsmith_book_model::matter::default_elements;
use wordsmith_book_model::{BookProject, Contributor, Edition, ElementKind, Imprint, OtherBook, Role, Source};
use wordsmith_manuscript_format::book::{self, FrontKind};
use wordsmith_manuscript_model::{Area, Block, Item, ItemId, ItemText, Kind, Place, Text};

fn fixture() -> wordsmith_manuscript_model::Project {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels/gullivers-travels.wsm");
    wordsmith_manuscript_format::folder::Folder::open(&root).unwrap().1
}

fn project() -> BookProject {
    let mut p = BookProject::new(Source::Manuscript("../Gulliver.wsm".into()), "2026-11-24");
    p.elements = default_elements();
    p.metadata.imprint = Some(Imprint {
        name: "Harbour Light Books".into(),
        address: vec!["1 Quay Street".into()],
        website: "https://example.org".into(),
        ..Imprint::default()
    });
    p.metadata.contributors.push(Contributor { name: "A. Editor".into(), role: Role::Editor });
    p.metadata.also_by = vec![
        OtherBook { title: "A Tale of a Tub".into(), series: String::new() },
        OtherBook { title: "A Modest Proposal".into(), series: String::new() },
    ];
    p.metadata.about_author = "Jonathan Swift was a satirist and a dean.".into();
    p.editions[0].isbn = "978-0-306-40615-7".into();
    p
}

fn text(blocks: &[Block]) -> Vec<String> {
    Text { blocks: blocks.to_vec(), notes: Vec::new() }.plain_paragraphs()
}

#[test]
fn each_element_is_made_from_the_metadata() {
    let mut src = fixture();
    src.settings.book.copyright_year = Some(1726);
    // The fixture has its own title page and "About the Author"; take them out to see Book's.
    let view = book::from_project(&src);
    src.binder.remove(view.front[0].id);
    src.binder.remove(view.back[0].id);
    let b = book::from_project(&src);
    let p = project();
    let a = assemble(&p, &b, p.edition("paperback"));
    let find = |k: ElementKind| {
        a.front.iter().chain(&a.back).find(|x| x.what == What::Element(k)).map(|x| text(&x.blocks)).unwrap_or_else(|| panic!("{k:?}"))
    };
    assert_eq!(find(ElementKind::HalfTitle), ["Gulliver’s Travels"]);
    let tp = find(ElementKind::TitlePage);
    assert_eq!(&tp[..3], ["Gulliver’s Travels", "into Several Remote Nations of the World", "Jonathan Swift"]);
    assert!(tp.contains(&"Edited by A. Editor".to_owned()) && tp.contains(&"Harbour Light Books".to_owned()));
    let c = find(ElementKind::Copyright);
    assert!(c.contains(&"Copyright © 1726 Jonathan Swift".to_owned()), "{c:?}");
    assert!(c.contains(&"This text is in the public domain.".to_owned()));
    assert!(c.contains(&"ISBN 978-0-306-40615-7 (paperback)".to_owned()), "{c:?}");
    assert!(c.contains(&"Published by Harbour Light Books".to_owned()) && c.contains(&"1 Quay Street".to_owned()));
    assert_eq!(find(ElementKind::AlsoBy), ["A Tale of a Tub", "A Modest Proposal"]);
    assert_eq!(find(ElementKind::AboutTheAuthor), ["Jonathan Swift was a satirist and a dean."]);
    // The ebook edition's copyright page has no print ISBN.
    let e = assemble(&p, &b, p.edition("ebook"));
    let ec = e.front.iter().find(|x| x.what == What::Element(ElementKind::Copyright)).map(|x| text(&x.blocks)).unwrap();
    assert!(!ec.iter().any(|l| l.contains("ISBN")), "{ec:?}");
    // The source's front matter follows Book's, in its own order; the body has parts then chapters.
    assert_eq!(a.front.iter().filter(|x| matches!(x.what, What::Matter(_))).count(), 2);
    assert_eq!(a.body.iter().filter(|x| x.what == What::Part).count(), 4);
    assert_eq!(a.body.iter().filter(|x| x.what == What::Chapter).count(), 39);
    assert!(a.unknown.is_empty());
}

#[test]
fn edited_elements_round_trip_through_the_format() {
    let mut p = project();
    let e = p.elements.iter_mut().find(|e| e.kind == Some(ElementKind::Copyright)).unwrap();
    e.text = "© {year} {holder}. {nonsense} Printed by {imprint}.".into();
    e.title = "The Small Print".into();
    let back = toml_file::read(&toml_file::write(&p).unwrap()).unwrap();
    assert_eq!(back, p);
    let mut src = fixture();
    src.settings.book.copyright_year = Some(1726);
    let a = assemble(&back, &book::from_project(&src), None);
    let c = a.front.iter().find(|x| x.what == What::Element(ElementKind::Copyright)).unwrap();
    assert_eq!(c.title, "The Small Print");
    assert_eq!(text(&c.blocks), ["© 1726 Jonathan Swift. {nonsense} Printed by Harbour Light Books."]);
    assert_eq!(a.unknown, ["nonsense"]);
}

#[test]
fn a_sources_own_pages_are_used_never_doubled() {
    let mut src = fixture();
    let id = ItemId::from_counter(9_000);
    let _ = src
        .binder
        .insert(Place::end_of(Area::Manuscript), Item::with_id(id, Kind::Text, "Copyright").role(wordsmith_manuscript_model::Role::FrontMatter));
    src.texts.insert(id, ItemText::with_body(Text::from_paragraphs(&["The writer's own copyright page."])));
    let b = book::from_project(&src);
    let a = assemble(&project(), &b, None);
    let copyrights: Vec<_> =
        a.front.iter().filter(|x| matches!(x.what, What::Element(ElementKind::Copyright) | What::Matter(FrontKind::Copyright))).collect();
    assert_eq!(copyrights.len(), 1);
    assert_eq!(copyrights[0].what, What::Matter(FrontKind::Copyright));
    assert_eq!(text(&copyrights[0].blocks), ["The writer's own copyright page."]);
    // It takes the place of Book's copyright page: after the title page, before the contents.
    let order: Vec<_> = a.front.iter().map(|x| x.what).collect();
    let at = |w: What| order.iter().position(|x| *x == w).unwrap();
    assert!(at(What::Matter(FrontKind::TitlePage)) < at(What::Matter(FrontKind::Copyright)));
    assert!(at(What::Matter(FrontKind::Copyright)) < at(What::Element(ElementKind::Contents)));
    // The fixture's own title page is used too, so Book's is not made.
    assert!(!order.contains(&What::Element(ElementKind::TitlePage)));
    let _ = Edition::paperback();
}

#[test]
fn parts_and_chapters_are_numbered_each_on_their_own() {
    use wordsmith_book_model::{NumberStyle, Numbering, SectionSettings};
    let b = book::from_project(&fixture());
    let mut p = project();
    let a = assemble(&p, &b, None);
    let parts: Vec<_> = a.body.iter().filter(|x| x.what == What::Part).map(|x| x.label.clone().unwrap()).collect();
    assert_eq!(parts, ["Part One", "Part Two", "Part Three", "Part Four"]);
    let chapters: Vec<_> = a.body.iter().filter(|x| x.what == What::Chapter).map(|x| x.label.clone().unwrap()).collect();
    assert_eq!(chapters.first().map(String::as_str), Some("Chapter One"));
    assert_eq!(chapters.last().map(String::as_str), Some("Chapter Thirty-Nine"));
    // Other styles, a label of the user's, a chapter left unnumbered, a subtitle and an epigraph.
    p.numbering = Numbering { chapters: NumberStyle::Roman, parts: NumberStyle::LabelFigures, part_label: "Voyage".into(), ..Numbering::default() };
    let first = b.body[0].children[0].id;
    p.sections.insert(
        first,
        SectionSettings {
            subtitle: "Of the Author".into(),
            epigraph: "All is well.".into(),
            epigraph_source: "A sailor".into(),
            unnumbered: true,
            ..SectionSettings::default()
        },
    );
    let a = assemble(&p, &b, None);
    assert_eq!(a.body[0].label.as_deref(), Some("Voyage 1"));
    assert_eq!(a.body[1].label, None);
    assert_eq!(a.body[1].subtitle, "Of the Author");
    assert_eq!(a.body[1].epigraph, Some(("All is well.".into(), "A sailor".into())));
    assert_eq!(a.body[2].label.as_deref(), Some("II"));
    // A title that is already a number line ("CHAPTER I.") gets no second one.
    let mut src = fixture();
    let ch = b.body[0].children[1].id;
    src.binder.get_mut(ch).unwrap().title = "CHAPTER II.".into();
    let a = assemble(&project(), &book::from_project(&src), None);
    assert_eq!(a.body[2].label, None);
}
