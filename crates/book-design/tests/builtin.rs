// SPDX-License-Identifier: MIT OR Apache-2.0
//! The built-in designs (plan M4.2, R3.1): every one reads and validates; every font it names is
//! bundled and may be embedded; every ornament and scene-break mark is in the font that prints it;
//! the large-print design sets its type at 16 pt and up.

use wordsmith_book_design::builtin;
use wordsmith_book_design::fonts::{self, Embedding, Role};

#[test]
fn thirteen_designs_validate_and_cover_the_genres() {
    let all = builtin::all();
    assert_eq!(all.len(), builtin::FILES.len());
    let ids: Vec<&str> = all.iter().map(|d| d.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "classic",
            "literary",
            "romance",
            "thriller",
            "fantasy",
            "non-fiction",
            "memoir",
            "historical",
            "young-adult",
            "horror",
            "cozy",
            "poetry",
            "large-print"
        ]
    );
    for (id, _) in builtin::FILES {
        assert_eq!(builtin::get(id).map(|d| d.id), Some(id.to_owned()));
    }
    for genre in
        ["literary", "romance", "thriller", "fantasy", "non-fiction", "classic", "memoir", "historical", "young-adult", "horror", "mystery", "poetry"]
    {
        assert!(all.iter().any(|d| d.genres.iter().any(|g| g == genre)), "{genre}");
    }
}

#[test]
fn every_font_is_bundled_embeddable_and_has_the_marks() {
    for d in builtin::all() {
        for role in [&d.body.font, &d.chapter.font] {
            let role = Role::from_key(role).unwrap_or_else(|| panic!("{}: {role}", d.id));
            for face in fonts::of_role(role) {
                assert!(matches!(fonts::embedding(face.bytes), Embedding::Installable | Embedding::Allowed), "{}: {role:?} {}", d.id, face.style);
            }
        }
        // Ornaments and scene-break marks print in the body font.
        let body = fonts::regular(Role::from_key(&d.body.font).unwrap()).unwrap();
        for c in d.chapter.ornament.chars().chain(d.scene_break.mark.chars()).filter(|c| !c.is_whitespace()) {
            assert!(fonts::has_char(body.bytes, c), "{}: the body font has no {c:?}", d.id);
        }
        // Small capitals and italic come from real faces: a body role has all four.
        assert_eq!(fonts::of_role(Role::from_key(&d.body.font).unwrap()).len(), 4, "{}", d.id);
    }
}

#[test]
fn the_large_print_design_is_large() {
    let d = builtin::get("large-print").unwrap();
    assert!(d.body.size >= 16.0, "{}", d.body.size);
    assert!(d.heads.size >= 12.0 && d.chapter.number_size >= 16.0 && d.chapter.title_size >= 16.0);
    assert!(!d.body.justify, "ragged right");
}
