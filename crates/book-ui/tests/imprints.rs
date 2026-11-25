// SPDX-License-Identifier: MIT OR Apache-2.0
//! Imprint profiles (plan R4.2, META-3): a book that joins an imprint takes its name, address,
//! website, logo and copyright wording; the title page shows the logo and the copyright page the
//! imprint's own wording, in print and in the ebook, and both still pass preflight.

use std::path::{Path, PathBuf};

use serde_json::json;
use wordsmith_book_model::{ElementKind, Severity};
use wordsmith_book_ui::{BookApp, commands};

const HOUSE: &str = r#"
schema = 1

[[imprints]]
id = "lantern"
name = "Lantern Press"
address = ["1 Harbour Street", "Port Town"]
website = "lantern.example"
logo = "logos/lantern.png"
copyright = "Text copyright © {year} {holder}. All rights reserved.\nFirst published by {imprint}\n{address}\n{isbn_line}"
design = "classic"
"#;

fn set_up(name: &str) -> (BookApp, PathBuf) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("house/logos")).unwrap();
    let house = dir.join("house/house.toml");
    std::fs::write(&house, HOUSE).unwrap();
    // A logo: 600 × 200 pixels, enough for 2 × 0.67 in at 300 pixels per inch.
    let img =
        image::RgbImage::from_fn(600, 200, |x, y| if (x / 50 + y / 50) % 2 == 0 { image::Rgb([20, 40, 90]) } else { image::Rgb([240, 235, 220]) });
    let mut png = std::io::Cursor::new(Vec::new());
    img.write_to(&mut png, image::ImageFormat::Png).unwrap();
    std::fs::write(dir.join("house/logos/lantern.png"), png.into_inner()).unwrap();
    let file = dir.join("tale.txt");
    std::fs::write(&file, "CHAPTER I.\n\nThe lamp burned low over the harbour, and the ships slept.\n").unwrap();
    let mut app = BookApp::new();
    app.command(commands::IMPORT, &json!({ "file": file.display().to_string(), "path": dir.join("T.wsbk").display().to_string() })).unwrap();
    (app, house)
}

#[test]
fn the_title_and_copyright_pages_come_from_the_imprint() {
    let (mut app, house) = set_up("imprint-pages");
    app.command(commands::HOUSE_JOIN, &json!({ "house": house.display().to_string(), "imprint": "lantern" })).unwrap();
    let b = app.book.as_ref().unwrap();
    let im = b.files.project.metadata.imprint.clone().unwrap();
    assert_eq!((im.name.as_str(), im.website.as_str(), im.logo.as_str()), ("Lantern Press", "lantern.example", "assets/imprint-logo.png"));
    assert!(b.files.assets.contains_key("imprint-logo.png") && b.files.project.decorative.contains(&im.logo));
    // The pages, as assembled.
    let src = b.source.as_ref().unwrap();
    let a = wordsmith_book_format::assemble::assemble(&b.files.project, &src.book, b.files.project.edition("paperback"));
    let piece = |k: ElementKind| a.pieces().find(|p| matches!(p.what, wordsmith_book_format::assemble::What::Element(x) if x == k)).unwrap().clone();
    let title = piece(ElementKind::TitlePage);
    assert!(
        title.blocks.iter().any(|bl| matches!(bl, wordsmith_manuscript_model::Block::Image { src, .. } if src == "assets/imprint-logo.png")),
        "the logo is on the title page"
    );
    let copyright: Vec<String> = piece(ElementKind::Copyright).blocks.iter().map(wordsmith_book_format::assemble::block_text).collect();
    assert!(copyright.iter().any(|l| l == "First published by Lantern Press"), "{copyright:?}");
    assert!(copyright.iter().any(|l| l == "1 Harbour Street") && copyright.iter().any(|l| l.contains("All rights reserved")), "{copyright:?}");
    // In print: the logo is a picture on the title page, the wording on the copyright page.
    let r = wordsmith_book_ui::preflight::run(b, "paperback", &Default::default()).unwrap();
    assert!(!r.findings.iter().any(|f| f.severity == Severity::Blocks && !["metadata", "isbn", "page-count"].contains(&f.rule)), "{:#?}", r.findings);
    let pr = wordsmith_book_check::build::print(&b.files.project, &b.files.designs, &src.book, b.files.project.edition("paperback").unwrap(), &|p| {
        b.picture(p)
    })
    .unwrap();
    let pdf = pr.typeset.pdf().unwrap();
    let limits = jcj_output::ReadLimits { max_input_bytes: 1 << 30, max_pages: 10_000, max_glyphs_per_page: 50_000 };
    let facts = jcj_output::read_facts(&pdf, &limits).unwrap();
    assert!(facts.pages.iter().take(4).any(|p| !p.images.is_empty()), "a picture on the opening pages");
    let text: String = jcj_output::read_pages(&pdf, &limits).unwrap().iter().flat_map(|p| p.glyphs.iter().filter_map(|g| g.text.clone())).collect();
    assert!(text.contains("FirstpublishedbyLanternPress") || text.contains("First published by Lantern Press"), "the wording is printed");
    // In the ebook: the logo is in the package and on the title page, hidden from screen readers.
    let (_, e) = wordsmith_book_check::build::ebook(&b.files.project, &b.files.designs, &src.book, b.files.project.edition("ebook").unwrap(), &|p| {
        b.picture(p)
    })
    .unwrap();
    let files = wordsmith_book_check::epub::unzip(&e.bytes).unwrap();
    let logo = b.files.assets.get("imprint-logo.png").unwrap();
    assert!(files.iter().any(|f| f.0.starts_with("EPUB/images/") && &f.2 == logo), "the logo is in the EPUB");
    let xhtml: String = files.iter().filter(|f| f.0.ends_with(".xhtml")).map(|f| String::from_utf8_lossy(&f.2).into_owned()).collect();
    assert!(xhtml.contains("First published by Lantern Press"));
    assert!(xhtml.contains("role=\"presentation\"><img"), "decorative: hidden from screen readers");
    let r = wordsmith_book_ui::preflight::run(b, "ebook", &Default::default()).unwrap();
    assert!(!r.findings.iter().any(|f| f.severity == Severity::Blocks && !["metadata", "isbn", "cover"].contains(&f.rule)), "{:#?}", r.findings);
}
