// SPDX-License-Identifier: MIT OR Apache-2.0
//! Proof PDFs (plan R3.6, PRT-13): a proof has "PROOF" across every page and its lines numbered
//! in the margin, breaks its pages as the edition does, and is never what export writes.

mod common;

use common::*;
use wordsmith_book_check::build;
use wordsmith_book_model::Edition;
use wordsmith_book_typeset::PROOF_MARK;

fn page_texts(pdf: &[u8]) -> Vec<(String, Vec<(f64, String)>)> {
    let limits = jcj_output::ReadLimits { max_input_bytes: 1 << 30, max_pages: 10_000, max_glyphs_per_page: 50_000 };
    jcj_output::read_pages(pdf, &limits)
        .unwrap()
        .iter()
        .map(|p| {
            let all: String = p.glyphs.iter().filter_map(|g| g.text.as_deref()).collect();
            let digits: Vec<(f64, String)> = p
                .glyphs
                .iter()
                .filter_map(|g| g.text.as_deref().filter(|t| t.chars().all(|c| c.is_ascii_digit()) && !t.is_empty()).map(|t| (g.x, t.to_owned())))
                .collect();
            (all, digits)
        })
        .collect()
}

#[test]
fn a_proof_is_marked_and_numbered_and_breaks_as_the_edition() {
    let book = gulliver();
    let e = Edition::paperback();
    let pr = build::print(&project(), &Default::default(), &book, &e, &|_| None).unwrap();
    let proof = build::proof(&project(), &Default::default(), &book, &e, &|_| None).unwrap();
    assert_eq!(proof.typeset.page_count(), pr.typeset.page_count(), "the same pages");
    assert_eq!(proof.typeset.author_lines(), pr.typeset.author_lines(), "the same lines");
    let (plain, marked) = (page_texts(&pr.typeset.pdf().unwrap()), page_texts(&proof.typeset.pdf().unwrap()));
    assert!(plain.iter().all(|(t, _)| !t.contains(PROOF_MARK)), "the edition carries no mark");
    let pages_marked = marked.iter().filter(|(t, _)| t.contains(PROOF_MARK)).count();
    assert_eq!(pages_marked, marked.len(), "every page of the proof says PROOF");
    // Line numbers: on a full page of text, 1 to n down the inside margin, left of the text.
    let (_, digits) = &marked[marked.len() / 2];
    let (_, plain_digits) = &plain[plain.len() / 2];
    // The reader gives a glyph per digit: the margin's digits read 1, 2, … n in order.
    let margin: String = digits.iter().filter(|d| !plain_digits.contains(d)).map(|d| d.1.as_str()).collect();
    let n = (1..200).find(|n| (1..=*n).map(|i: u32| i.to_string()).collect::<String>() == margin);
    assert!(n.is_some_and(|n| n >= 20), "the margin reads {margin}");
    // Never declares the standards, even for an edition that asks.
    let mut standards = e.clone();
    standards.pdf_standards = true;
    let p2 = build::proof(&project(), &Default::default(), &book, &standards, &|_| None).unwrap();
    assert!(!wordsmith_book_check::pdf_check::declares(&p2.typeset.pdf().unwrap()).is_empty());
}

#[test]
fn export_never_writes_a_proof() {
    // Export writes what preflight builds and checks: the edition's own PDF, unmarked.
    let src = wordsmith_book_format::source::open(&novels(), &wordsmith_book_model::Source::Manuscript("gullivers-travels.wsm".into())).unwrap();
    let (_, bytes) = wordsmith_book_check::preflight::preflight(&project(), &Default::default(), &src, &Edition::paperback(), &|_| None);
    let bytes = bytes.unwrap();
    assert!(page_texts(&bytes).iter().all(|(t, _)| !t.contains(PROOF_MARK)));
}
