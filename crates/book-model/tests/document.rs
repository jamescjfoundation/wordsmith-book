// SPDX-License-Identifier: MIT OR Apache-2.0
//! Book's text ↔ document conversion (plan M1.5): every block and mark goes text → document →
//! text exactly (property tests; every section of the Manuscript fixture in book-format's tests),
//! and a document with formatting the book view cannot hold reports what it dropped.

use proptest::prelude::*;
use wordsmith_book_model::document::{from_document, to_document};
use wordsmith_manuscript_model::{Block, Fence, ImageRole, Inline, Marks, Note, Text};

fn word() -> impl Strategy<Value = String> {
    "[A-Za-z’é—,.;: ]{1,12}"
}

fn marks() -> impl Strategy<Value = Marks> {
    (
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        proptest::option::of("[a-z]{2}"),
        proptest::option::of("https://example\\.org/[a-z]{1,5}"),
        proptest::option::of("c[0-9]"),
        proptest::option::of(0u32..3),
    )
        .prop_map(|(italic, bold, small_caps, strike, lang, link, comment, rev)| Marks {
            italic,
            bold,
            small_caps,
            strike,
            lang,
            link,
            comment,
            rev,
        })
}

fn spans() -> impl Strategy<Value = Vec<Inline>> {
    proptest::collection::vec((word(), marks()).prop_map(|(t, m)| Inline::marked(t, m)), 0..5)
}

fn verse_spans() -> impl Strategy<Value = Vec<Inline>> {
    proptest::collection::vec(word(), 1..4).prop_map(|lines| vec![Inline::plain(lines.join("\n"))])
}

fn block() -> impl Strategy<Value = Block> {
    prop_oneof![
        spans().prop_map(Block::Para),
        (1u8..4, spans()).prop_map(|(level, content)| Block::Heading { level, content }),
        Just(Block::SceneBreak),
        proptest::collection::vec(spans(), 1..3).prop_map(Block::Quote),
        proptest::collection::vec(verse_spans(), 1..3).prop_map(|paras| Block::Fenced { kind: Fence::Verse, paras }),
        (proptest::option::of("[a-z]{3,6}"), proptest::collection::vec(spans(), 1..3))
            .prop_map(|(k, paras)| Block::Fenced { kind: Fence::Aside(k), paras }),
        (word(), "media/[a-z]{4}\\.png", 0..3u8).prop_map(|(alt, src, r)| Block::Image {
            alt,
            src,
            role: [ImageRole::Inline, ImageRole::Wide, ImageRole::Page][r as usize]
        }),
        ("[a-z]{3,5}", word()).prop_map(|(format, text)| Block::Raw { format, text }),
    ]
}

prop_compose! {
    fn text()(blocks in proptest::collection::vec(block(), 0..8), notes in proptest::collection::vec(spans(), 0..3), unreferenced in proptest::collection::vec(spans(), 0..2)) -> Text {
        let mut t = Text { blocks, notes: Vec::new() };
        // Reference each note once, at the end of the first paragraph (or a new one).
        for (i, paras) in notes.into_iter().enumerate() {
            let label = format!("{}", i + 1);
            let r = Inline::NoteRef { label: label.clone(), marks: Marks::default() };
            match t.blocks.iter_mut().find(|b| matches!(b, Block::Para(_))) {
                Some(Block::Para(c)) => c.push(r),
                _ => t.blocks.push(Block::Para(vec![Inline::plain("See"), r])),
            }
            t.notes.push(Note { label, paras: vec![paras] });
        }
        for (i, paras) in unreferenced.into_iter().enumerate() {
            t.notes.push(Note { label: format!("u{i}"), paras: vec![paras] });
        }
        t.normalize();
        t
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn every_block_and_mark_goes_text_document_text_exactly(t in text()) {
        let (back, dropped) = from_document(&to_document(&t));
        prop_assert!(dropped.is_empty(), "nothing dropped: {:?}", dropped);
        prop_assert_eq!(back, t);
    }
}
