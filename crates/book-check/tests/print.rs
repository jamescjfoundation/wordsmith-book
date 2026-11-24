// SPDX-License-Identifier: MIT OR Apache-2.0
//! The full book in print (plan M5.1): the test books typeset, their PDFs read back letter by
//! letter; the author's words checked against the source; text that looks like markup printed
//! literally; and the page map.

mod common;

use common::*;
use wordsmith_book_typeset::from_book::source_words;
use wordsmith_book_typeset::{Block, Kind, PrintBook, Section, Span, typeset};

fn check(name: &str, b: &wordsmith_manuscript_format::book::Book) {
    let (pb, _, t) = print(b, &classic());
    let pdf = t.pdf().unwrap();
    // Every letter the document holds is printed, in order.
    let printed: String = t.printed_paragraphs().iter().map(|p| p.0.as_str()).collect::<Vec<_>>().join(" ");
    let want = letters(&printed);
    let got = letters(&pdf_body_text(&t, &pdf));
    if got != want {
        let at = got.chars().zip(want.chars()).position(|(a, b)| a != b).unwrap_or(got.len().min(want.len()));
        let around = |s: &str| s.chars().skip(at.saturating_sub(40)).take(80).collect::<String>();
        panic!("{name}: letters differ at {at} of {}: printed `{}`, document `{}`", want.len(), around(&got), around(&want));
    }
    // The author's words, in order, are the source's.
    let src = wordsmith_book_check_words(b);
    let ours = source_words(&pb);
    if let Some(at) = ours.iter().zip(&src).position(|(a, b)| a != b) {
        panic!(
            "{name}: word {at}: print has {:?}, source {:?}",
            &ours[at.saturating_sub(3)..(at + 3).min(ours.len())],
            &src[at.saturating_sub(3)..(at + 3).min(src.len())]
        );
    }
    assert_eq!(ours.len(), src.len(), "{name}");
    // The page map points every section at the page where it starts.
    let map = t.page_map();
    assert_eq!(map.len(), pb.sections.len(), "{name}");
    let pieces = jcj_layout::pages::body_pieces(t.layout());
    for (s, start) in pb.sections.iter().zip(&map) {
        assert_eq!(s.id, start.id);
        assert!(start.page >= 1 && start.page <= t.page_count());
        assert!(pieces[start.page - 1].iter().any(|p| !p.continued), "{name}: {} starts on page {}", s.id, start.page);
    }
    assert!(map.windows(2).all(|w| w[0].page <= w[1].page), "{name}: sections in page order");
}

/// The book view's words in reading order with notes (the word rule of the core's check).
fn wordsmith_book_check_words(b: &wordsmith_manuscript_format::book::Book) -> Vec<String> {
    fn walk(s: &wordsmith_manuscript_format::book::Section, out: &mut Vec<String>) {
        let w = |t: &str| t.split(char::is_whitespace).filter(|w| w.chars().any(char::is_alphanumeric)).map(str::to_owned).collect::<Vec<_>>();
        if matches!(s.role, wordsmith_manuscript_model::Role::Part | wordsmith_manuscript_model::Role::Chapter) {
            out.extend(w(&s.title));
        }
        let t = wordsmith_manuscript_model::Text { blocks: s.content.clone(), notes: s.notes.clone() };
        for p in t.plain_paragraphs() {
            out.extend(w(&p));
        }
        for c in &s.children {
            walk(c, out);
        }
    }
    let mut out = Vec::new();
    for s in b.front.iter().chain(&b.body).chain(&b.back) {
        walk(s, &mut out);
    }
    out
}

#[test]
fn gullivers_travels_prints_every_letter_and_every_word() {
    check("Gulliver's Travels", &gulliver());
}

#[test]
fn waverley_prints_every_letter_and_every_word() {
    check("Waverley", &waverley());
}

#[test]
fn text_that_looks_like_markup_prints_literally() {
    let tricky = "#set page(width: 1in) {PAGE} <b>bold</b> *emph* $x^2$ \\par {\\rtf1} %% [[link]] @{field} -- done";
    let book = PrintBook {
        title: "T".into(),
        author: "A".into(),
        lang: "en".into(),
        sections: vec![Section {
            id: "s".into(),
            kind: Kind::Chapter,
            title: "{TITLE}".into(),
            blocks: vec![Block::Para(vec![Span::plain(tricky)])],
            ..Section::default()
        }],
    };
    let look = wordsmith_book_typeset::from_book::look(&classic(), &Default::default()).unwrap();
    let t = typeset(&book, &look, "2026-11-24").unwrap();
    let pdf = t.pdf().unwrap();
    // The first words are small capitals, which read back as capitals.
    let text = pdf_body_text(&t, &pdf).replace(char::is_whitespace, "").to_lowercase();
    assert!(text.contains(&tricky.replace(' ', "").to_lowercase()), "printed: {text}");
    assert!(text.contains("{title}"));
}

/// A picture written for the test: a 600 × 300 pixel PNG.
fn png() -> Vec<u8> {
    let img = image::RgbImage::from_fn(600, 300, |x, y| image::Rgb([(x % 256) as u8, (y % 256) as u8, 128]));
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

#[test]
fn pictures_print_inline_and_wide_with_their_alternative_text() {
    let bytes = png();
    assert_eq!(wordsmith_book_typeset::from_book::image_size(&bytes), Some((600, 300)));
    let pic = |fit| Block::Image { bytes: bytes.clone(), pixels: (600, 300), alt: "A lighthouse at dusk".into(), fit };
    let book = PrintBook {
        title: "T".into(),
        author: "A".into(),
        lang: "en".into(),
        sections: vec![Section {
            id: "s".into(),
            kind: Kind::Chapter,
            title: "Pictures".into(),
            blocks: vec![
                Block::Para(vec![Span::plain("Before.")]),
                pic(wordsmith_book_typeset::ImageFit::Inline),
                pic(wordsmith_book_typeset::ImageFit::Wide),
                Block::Para(vec![Span::plain("After.")]),
            ],
            ..Section::default()
        }],
    };
    let look = wordsmith_book_typeset::from_book::look(&classic(), &Default::default()).unwrap();
    let t = typeset(&book, &look, "2026-11-24").unwrap();
    let doc = t.document();
    assert_eq!(doc.media.len(), 2);
    let images: Vec<(f32, String)> = jcj_doc::walk::body_paragraphs(doc)
        .iter()
        .flat_map(|(_, p)| {
            p.objects_with_offsets()
                .filter_map(|(_, o)| if let jcj_doc::para::InlineObject::Image { w, alt, .. } = o { Some((*w, alt.clone())) } else { None })
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(images.len(), 2);
    assert!(images.iter().all(|(_, alt)| alt == "A lighthouse at dusk"), "the alternative text is carried for tagging");
    assert!((images[0].0 - 144.0).abs() < 0.5, "inline: 600 pixels at 300 per inch is 2 in");
    assert!(images[1].0 > 250.0, "wide: the width of the text");
    assert!(t.pdf().unwrap().len() > 5000);
}
