// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bleed and full-page pictures (plan R2.3, PRT-7, TXT-4): an edition that bleeds has pages larger
//! by the bleed outside, top and bottom, with trim and bleed boxes; a full-page picture fills its
//! page to the edge; without bleed it stays inside the text block; preflight passes both.

mod common;

use common::*;
use wordsmith_book_check::{build, preflight};
use wordsmith_book_format::source;
use wordsmith_book_model::{Edition, Severity, Source};
use wordsmith_manuscript_model::{Block, ImageRole, Role};

fn png(w: u32, h: u32) -> Vec<u8> {
    let img = image::RgbImage::from_fn(w, h, |x, y| image::Rgb([(x % 251) as u8, (y % 241) as u8, 70]));
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

fn bleeding() -> Edition {
    let mut e = Edition::paperback();
    e.bleed = true;
    e
}

fn with_full_page_picture() -> wordsmith_manuscript_format::book::Book {
    fn first(s: &mut [wordsmith_manuscript_format::book::Section]) -> Option<&mut wordsmith_manuscript_format::book::Section> {
        for x in s {
            if x.role == Role::Chapter && !x.content.is_empty() {
                return Some(x);
            }
            if let Some(c) = first(&mut x.children) {
                return Some(c);
            }
        }
        None
    }
    let mut b = gulliver();
    first(&mut b.body)
        .unwrap()
        .content
        .insert(2, Block::Image { alt: "A map of the island".into(), src: "media/map.png".into(), role: ImageRole::Page });
    b
}

#[test]
fn pages_that_bleed_carry_trim_and_bleed_boxes() {
    let e = bleeding();
    let pr = build::print(&project(), &Default::default(), &gulliver(), &e, &|_| None).unwrap();
    let bleed = f64::from(pr.profile.bleed) * 72.0;
    assert!(bleed > 0.0);
    let (tw, th) = (f64::from(pr.look.trim.0) * 72.0, f64::from(pr.look.trim.1) * 72.0);
    let pdf = pr.typeset.pdf().unwrap();
    let facts = jcj_output::read_facts(&pdf, &PDF_LIMITS).unwrap();
    for (i, p) in facts.pages.iter().enumerate() {
        assert!((p.media.width() - (tw + bleed)).abs() < 0.05 && (p.media.height() - (th + 2.0 * bleed)).abs() < 0.05, "page {}", i + 1);
        let t = p.trim.unwrap();
        assert!((t.width() - tw).abs() < 0.05 && (t.height() - th).abs() < 0.05);
        assert_eq!(p.bleed, Some(p.media));
        // The bleed is outside: right on right-hand pages, left on left-hand ones.
        let outside = if i % 2 == 0 { p.media.x1 - t.x1 } else { t.x0 - p.media.x0 };
        assert!((outside - bleed).abs() < 0.05 && (t.y0 - bleed).abs() < 0.05, "page {}", i + 1);
    }
    // Margins are kept from the trim, so the text sits where it would without bleed.
    let plain = build::print(&project(), &Default::default(), &gulliver(), &Edition::paperback(), &|_| None).unwrap();
    assert_eq!(plain.typeset.page_count(), pr.typeset.page_count());
    let (x, y, w, h) = pr.typeset.body_area(0).unwrap();
    assert_eq!((x, y - bleed as f32, w, h), {
        let (x, y, w, h) = plain.typeset.body_area(0).unwrap();
        (x, y, w, h)
    });
    // Preflight passes the edition that bleeds.
    let src = source::open(&novels(), &Source::Manuscript("gullivers-travels.wsm".into())).unwrap();
    let (r, _) = preflight::preflight(&project(), &Default::default(), &src, &e, &|_| None);
    assert!(!r.blocked(), "{:#?}", r.of(Severity::Blocks));
    // A PDF without its boxes is caught when the edition bleeds.
    let mut stripped = pr;
    let unboxed = {
        let mut no = stripped.look.clone();
        no.bleed = 0.0;
        let t = wordsmith_book_typeset::typeset(&stripped.book, &no, "2026-11-24").unwrap();
        t.pdf().unwrap()
    };
    stripped.look.bleed = if e.bleed { stripped.profile.bleed } else { 0.0 };
    assert!(preflight::pdf_facts(&stripped, &unboxed).iter().any(|f| f.rule == "page-size" && f.severity == Severity::Blocks));
}

#[test]
fn a_full_page_picture_fills_its_page_with_bleed_and_the_text_block_without() {
    let b = with_full_page_picture();
    let pic = png(2400, 3600);
    for e in [bleeding(), Edition::paperback()] {
        let pr = build::print(&project(), &Default::default(), &b, &e, &|_| Some(pic.clone())).unwrap();
        let pdf = pr.typeset.pdf().unwrap();
        let facts = jcj_output::read_facts(&pdf, &PDF_LIMITS).unwrap();
        let (i, page) = facts.pages.iter().enumerate().find(|(_, p)| !p.images.is_empty()).expect("the picture is printed");
        let im = page.images[0];
        if e.bleed {
            assert!(im.placed_width >= page.media.width() - 1.0 && im.placed_height >= page.media.height() - 1.0, "{im:?} on {:?}", page.media);
        } else {
            let (_, _, w, h) = pr.typeset.body_area(i).unwrap();
            assert!(im.placed_width <= f64::from(w) + 1.0 && im.placed_height <= f64::from(h) + 1.0, "{im:?}");
        }
        // The picture's page holds nothing of the text; the words all print.
        let pages = jcj_output::read_pages(&pdf, &PDF_LIMITS).unwrap();
        let lines = page_lines(&pages[i]);
        assert!(lines.len() <= 2, "page {} has text: {lines:?}", i + 1);
        assert!(preflight::print_words(&pr, &pdf, &b).is_empty());
        assert!(im.ppi() >= 300.0, "{}", im.ppi());
    }
}
