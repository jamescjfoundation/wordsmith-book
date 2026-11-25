// SPDX-License-Identifier: MIT OR Apache-2.0
//! PDF standards (plan R3.5, PRT-12): a print edition set to declare PDF/A-2b and PDF/UA-1 gets
//! a tagged PDF with both declarations (core v0.1.15) and the same text and facts as without;
//! preflight blocks an edition whose PDF lacks them; where the standard PDF validator is
//! installed (`BOOK_PDF_VALIDATOR`, with `BOOK_JAVA`), the test book's PDF passes both.

mod common;

use common::*;
use wordsmith_book_check::{build, pdf_check, preflight, validator};
use wordsmith_book_model::{Edition, Severity};

fn standards() -> Edition {
    let mut e = Edition::paperback();
    e.pdf_standards = true;
    e
}

#[test]
fn an_edition_with_the_standards_declares_both_and_prints_the_same() {
    let book = gulliver();
    let plain = build::print(&project(), &Default::default(), &book, &Edition::paperback(), &|_| None).unwrap();
    let pr = build::print(&project(), &Default::default(), &book, &standards(), &|_| None).unwrap();
    let (a, b) = (plain.typeset.pdf().unwrap(), pr.typeset.pdf().unwrap());
    assert!(pdf_check::declares(&a).len() == 2 && pdf_check::declares(&b).is_empty());
    let limits = jcj_output::ReadLimits { max_input_bytes: 1 << 30, max_pages: 10_000, max_glyphs_per_page: 50_000 };
    assert_eq!(jcj_output::read_pages(&a, &limits).unwrap(), jcj_output::read_pages(&b, &limits).unwrap(), "the same text");
    assert_eq!(jcj_output::read_facts(&a, &limits).unwrap().fonts, jcj_output::read_facts(&b, &limits).unwrap().fonts);
    let text = String::from_utf8_lossy(&b);
    assert!(text.contains("<rdf:li>Jonathan Swift</rdf:li>") && text.contains("/Lang (en)"), "the author and the book's language");
    assert_eq!(b, pr.typeset.pdf().unwrap(), "the same bytes every time");
    // Preflight: nothing blocks it; the same file without the declarations is blocked.
    let f = preflight::print_findings(&pr, &b, &book, &standards());
    assert!(!f.iter().any(|x| x.rule == "pdf-standards"), "{f:#?}");
    let f = preflight::print_findings(&pr, &a, &book, &standards());
    assert_eq!(f.iter().filter(|x| x.rule == "pdf-standards" && x.severity == Severity::Blocks).count(), 2);
    // Not set up: a note.
    let notes = pdf_check::check(&validator::Settings::default(), &b);
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].severity, Severity::Note);
}

#[test]
fn the_standard_validator_passes_the_test_book_where_it_is_installed() {
    let Some(program) = std::env::var_os("BOOK_PDF_VALIDATOR") else {
        eprintln!("no PDF validator is set up (BOOK_PDF_VALIDATOR); skipped");
        return;
    };
    let pr = build::print(&project(), &Default::default(), &gulliver(), &standards(), &|_| None).unwrap();
    let s = validator::Settings {
        pdf: program.to_string_lossy().into_owned(),
        java: std::env::var("BOOK_JAVA").unwrap_or_default(),
        seconds: 900,
        ..validator::Settings::default()
    };
    let f = pdf_check::check(&s, &pr.typeset.pdf().unwrap());
    assert!(f.is_empty(), "{f:#?}");
}
