// SPDX-License-Identifier: MIT OR Apache-2.0
//! The printed cover (plan R2.11, PRT-9, PRT-10): the full-wrap template's size for page counts
//! and papers, its spine text rule and barcode area; good and bad covers checked against it.

use wordsmith_book_check::cover::{self, MIN_PPI};
use wordsmith_book_check::profile;
use wordsmith_book_model::{Paper, Severity};

fn png(w: u32, h: u32) -> Vec<u8> {
    let img = image::RgbImage::from_pixel(w, h, image::Rgb([30, 60, 90]));
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

#[test]
fn the_template_follows_the_page_count_and_paper() {
    let pod = profile::get("pod-paperback").unwrap();
    for (pages, paper, per_page) in [(100, Paper::White, 0.002252), (300, Paper::Cream, 0.0025), (240, Paper::Color, 0.002347)] {
        let t = cover::template(&pod, (6.0, 9.0), pages, paper);
        let spine = pages as f64 * per_page;
        assert!((t.spine - spine).abs() < 1e-9, "{pages} {paper:?}");
        assert!((t.width - (2.0 * 0.125 + 12.0 + spine)).abs() < 1e-9);
        assert!((t.height - 9.25).abs() < 1e-9);
        assert!((t.front.x - (0.125 + 6.0 + spine)).abs() < 1e-9 && (t.spine_area.w - spine).abs() < 1e-9);
        assert!(cover::barcode_fits(&t));
        // Spine text only from the profile's page count.
        assert_eq!(t.spine_safe.is_some(), pages >= pod.spine_text_min_pages, "{pages}");
    }
    let thin = cover::template(&pod, (5.0, 8.0), 60, Paper::Cream);
    assert!(thin.spine_safe.is_none());
    // A hardcover's profile gives its own sizes.
    let hard = profile::get("pod-hardcover").unwrap();
    let t = cover::template(&hard, (6.0, 9.0), 300, Paper::Cream);
    assert!((t.spine - 300.0 * 0.0025).abs() < 1e-9);
}

#[test]
fn the_template_files_have_the_covers_size() {
    let pod = profile::get("pod-paperback").unwrap();
    let t = cover::template(&pod, (5.5, 8.5), 200, Paper::Cream);
    let pdf = cover::template_pdf(&t);
    assert_eq!(pdf, cover::template_pdf(&t), "the same bytes every time");
    let facts = jcj_output::read_facts(&pdf, &jcj_output::ReadLimits::default()).unwrap();
    assert_eq!(facts.pages.len(), 1);
    assert!((facts.pages[0].media.width() - t.width * 72.0).abs() < 0.01 && (facts.pages[0].media.height() - t.height * 72.0).abs() < 0.01);
    let guide = cover::template_png(&t, 100.0);
    let (w, h) = wordsmith_book_model::picture::size(&guide).unwrap();
    assert_eq!((w, h), ((t.width * 100.0).round() as u32, (t.height * 100.0).round() as u32));
    // The template PDF itself passes as a cover of that size.
    assert!(cover::check(&pdf, &t).is_empty());
}

#[test]
fn good_and_bad_covers() {
    let pod = profile::get("pod-paperback").unwrap();
    let t = cover::template(&pod, (6.0, 9.0), 320, Paper::Cream);
    let at = |ppi: f64| ((t.width * ppi).ceil() as u32, (t.height * ppi).ceil() as u32);
    let (w, h) = at(MIN_PPI);
    assert!(cover::check(&png(w, h), &t).is_empty(), "an image at 300 pixels per inch, the right shape");
    // Too few pixels.
    let (w2, h2) = at(150.0);
    let low = cover::check(&png(w2, h2), &t);
    assert!(low.iter().any(|f| f.severity == Severity::Blocks && f.message.contains("pixels per inch")), "{low:?}");
    // The wrong shape: a cover made for another page count.
    let other = cover::template(&pod, (6.0, 9.0), 80, Paper::Cream);
    let (w3, h3) = ((other.width * 300.0).ceil() as u32, (other.height * 300.0).ceil() as u32);
    assert!(cover::check(&png(w3, h3), &t).iter().any(|f| f.message.contains("not the shape")));
    // A PDF for another page count.
    let wrong = cover::check(&cover::template_pdf(&other), &t);
    assert!(wrong.iter().any(|f| f.message.contains("must be")), "{wrong:?}");
    // Not a cover at all.
    assert!(cover::check(b"hello", &t).iter().any(|f| f.severity == Severity::Blocks));
}
