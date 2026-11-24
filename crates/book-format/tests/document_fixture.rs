// SPDX-License-Identifier: MIT OR Apache-2.0
//! Book's text ↔ document conversion (plan M1.5) on every section of the Manuscript fixture.

use std::path::Path;

use wordsmith_book_model::document::{from_document, to_document};
use wordsmith_manuscript_model::Text;

#[test]
fn every_section_of_the_manuscript_fixture_converts_exactly() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels/gullivers-travels.wsm");
    let (_, project) = wordsmith_manuscript_format::folder::Folder::open(&root).unwrap();
    let book = wordsmith_manuscript_format::book::from_project(&project);
    let mut n = 0;
    let mut stack: Vec<_> = book.front.iter().chain(&book.body).chain(&book.back).collect();
    while let Some(s) = stack.pop() {
        stack.extend(s.children.iter());
        let mut t = Text { blocks: s.content.clone(), notes: s.notes.clone() };
        t.normalize();
        let (back, dropped) = from_document(&to_document(&t));
        assert!(dropped.is_empty());
        assert_eq!(back, t, "section {}", s.title);
        n += 1;
    }
    assert!(n > 40);
}
