// SPDX-License-Identifier: MIT OR Apache-2.0
//! The Book project format (plan M1.1): round trips, saving only what changed, the same bytes
//! twice, random projects, hostile input at every limit, schema versions, and every example in
//! `docs/format.md`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use proptest::prelude::*;
use wordsmith_book_format::{Error, Files, Folder, limits, toml_file};
use wordsmith_book_model::{
    BookProject, Contributor, DesignChoice, DesignPart, Edition, Element, ElementKind, Fix, Format, Imprint, Metadata, NoteStyle, Only, Paper,
    Placement, Role, SectionId, SectionSettings, Source,
};

fn tmp(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("format").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn full() -> BookProject {
    let mut p = BookProject::new(Source::Manuscript("../The Lighthouse.wsm".into()), "2026-11-24");
    p.design = DesignChoice { id: "classic".into(), mix: BTreeMap::from([(DesignPart::ChapterOpening, "fantasy".into())]) };
    p.metadata = Metadata {
        contributors: vec![Contributor { name: "A. Translator".into(), role: Role::Translator }],
        subject_codes: vec!["FIC019000".into()],
        imprint: Some(Imprint { name: "Harbour Light Books".into(), address: vec!["1 Quay Street".into()], ..Imprint::default() }),
        copyright_holder: "The Estate".into(),
        ..Metadata::default()
    };
    p.editions[0].isbn = "978-0-306-40615-7".into();
    p.editions[0].overrides.insert("body.size".into(), "11.5".into());
    p.editions.push(Edition {
        id: "large".into(),
        name: "Large print".into(),
        format: Format::LargePrint,
        paper: Some(Paper::White),
        ..Edition::default()
    });
    p.editions[1].notes = NoteStyle::ChapterEnd;
    p.editions[1].embed_fonts = false;
    p.elements.push(Element { id: "ded".into(), kind: Some(ElementKind::Dedication), text: "For the keepers.".into(), ..Element::default() });
    p.elements.push(Element {
        id: "news".into(),
        kind: Some(ElementKind::Newsletter),
        placement: Placement::Back,
        only: Only::Ebook,
        ..Element::default()
    });
    p.sections.insert(
        SectionId::from_counter(1),
        SectionSettings { subtitle: "In Which".into(), epigraph: "The sea.".into(), ..SectionSettings::default() },
    );
    p.fixes.push(Fix {
        id: "fix-1".into(),
        section: Some(SectionId::from_counter(1)),
        find: "teh harbour".into(),
        replace: "the harbour".into(),
        ..Fix::default()
    });
    p
}

fn files() -> Files {
    let mut f = Files::new(full());
    f.designs.insert("mine".into(), "schema = 1\nname = \"Mine\"\n".into());
    f.assets.insert("cover.png".into(), vec![0x89, b'P', b'N', b'G', 1, 2, 3]);
    f.builds.insert("paperback".into(), "{\"edition\":\"paperback\"}".into());
    f
}

#[test]
fn files_round_trip_exactly_and_give_the_same_bytes_twice() {
    let f = files();
    let a = f.render().unwrap();
    let back = Files::from_map(&a).unwrap();
    assert_eq!(back, f);
    assert_eq!(back.render().unwrap(), a);
}

#[test]
fn a_folder_saves_only_what_changed_and_reopens_the_same() {
    let dir = tmp("save").join("My Book.wsbk");
    let f = files();
    let mut folder = Folder::create(&dir, &f).unwrap();
    let (_, back) = Folder::open(&dir).unwrap();
    assert_eq!(back, f);
    let before = std::fs::read(dir.join("book.toml")).unwrap();
    // Saving again writes nothing.
    assert!(folder.save(&f).unwrap().written.is_empty());
    assert_eq!(std::fs::read(dir.join("book.toml")).unwrap(), before);
    // One design changes: one file is written. One asset goes: it is removed.
    let mut g = f.clone();
    g.designs.insert("mine".into(), "schema = 1\nname = \"Mine, again\"\n".into());
    g.assets.clear();
    let saved = folder.save(&g).unwrap();
    assert_eq!(saved.written, vec!["designs/mine.toml".to_owned()]);
    assert_eq!(saved.removed, vec!["assets/cover.png".to_owned()]);
    assert!(!dir.join("assets/cover.png").exists());
    // book.toml is written last when it changes.
    let mut h = g.clone();
    h.project.date = "2026-12-01".into();
    h.builds.insert("ebook".into(), "{}".into());
    let saved = folder.save(&h).unwrap();
    assert_eq!(saved.written.last().map(String::as_str), Some("book.toml"));
    assert_eq!(Folder::open(&dir).unwrap().1, h);
}

#[test]
fn a_project_saved_twice_gives_the_same_bytes() {
    let a = tmp("twice-a").join("A.wsbk");
    let b = tmp("twice-b").join("A.wsbk");
    Folder::create(&a, &files()).unwrap();
    Folder::create(&b, &files()).unwrap();
    for rel in ["book.toml", "designs/mine.toml", "assets/cover.png", "builds/paperback.json"] {
        assert_eq!(std::fs::read(a.join(rel)).unwrap(), std::fs::read(b.join(rel)).unwrap(), "{rel}");
    }
}

#[test]
fn a_newer_schema_is_refused_clearly() {
    let text = toml_file::write(&full()).unwrap().replace("schema = 1", "schema = 2");
    let e = toml_file::read(&text).unwrap_err();
    assert_eq!(e, Error::Newer { found: 2, supported: 1 });
    assert!(e.to_string().contains("newer version of Wordsmith Book"));
}

#[test]
fn version_0_migrates_to_version_1() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/projects/version-0.wsbk");
    let (_, f) = Folder::open(&root).unwrap();
    assert_eq!(f.project.schema, 1);
    assert_eq!(f.project.source, Source::Manuscript("../The Lighthouse.wsm".into()));
    assert_eq!(f.project.design.id, "literary");
    assert!(toml_file::write(&f.project).unwrap().contains("schema = 1"));
}

fn hostile(text: &str) -> Error {
    toml_file::read(text).unwrap_err()
}

#[test]
fn hostile_book_toml_fails_cleanly_at_every_limit() {
    let base = toml_file::write(&full()).unwrap();
    // Size.
    let big = format!("{base}\n# {}\n", "x".repeat(limits::MAX_BOOK_TOML as usize));
    assert!(matches!(hostile(&big), Error::TooLarge { .. }));
    // Nesting.
    let deep = format!("schema = 1\nx = {}{}\n", "[".repeat(100), "]".repeat(100));
    assert!(matches!(hostile(&deep), Error::Damaged { .. }));
    // Counts.
    let mut many = String::from("schema = 1\n[source]\nowned = \"source.wsm\"\n");
    for i in 0..=limits::MAX_EDITIONS {
        many.push_str(&format!("[[editions]]\nid = \"e{i}\"\nname = \"E\"\nformat = \"ebook\"\n"));
    }
    assert!(matches!(hostile(&many), Error::TooLarge { .. }));
    // No schema, a schema that is not a number, a negative one.
    assert_eq!(hostile("title = \"x\"\n"), Error::NotABook);
    assert_eq!(hostile("schema = \"one\"\n"), Error::NotABook);
    assert_eq!(hostile("schema = -1\n"), Error::NotABook);
    // Not TOML, wrong shapes, duplicate ids, a bad source.
    assert!(matches!(hostile("schema = 1\n[source\n"), Error::Damaged { .. }));
    assert!(matches!(hostile("schema = 1\n[source]\nowned = 3\n"), Error::Damaged { .. }));
    let dup = format!("{base}\n[[editions]]\nid = \"paperback\"\nname = \"Again\"\nformat = \"paperback\"\n");
    assert!(matches!(toml_file::read(&dup).unwrap_err(), Error::Damaged { .. }));
    assert!(matches!(hostile("schema = 1\n[source]\nowned = \"../elsewhere\"\n"), Error::Source(_)));
    assert!(matches!(hostile("schema = 1\n[source]\nmanuscript = \"notes.txt\"\n"), Error::Source(_)));
    let bad_fix = format!("{base}\n[[fixes]]\nid = \"f2\"\nfind = \"two\\nlines\"\nreplace = \"x\"\n");
    assert!(matches!(toml_file::read(&bad_fix).unwrap_err(), Error::Damaged { .. }));
}

#[test]
fn hostile_folders_fail_cleanly() {
    let good = files().render().unwrap();
    let with = |k: &str, v: &[u8]| {
        let mut m = good.clone();
        m.insert(k.into(), v.to_vec());
        Files::from_map(&m)
    };
    assert!(matches!(with("book.toml", &[0xff, 0xfe, 0x00]), Err(Error::Damaged { .. })));
    assert!(matches!(with("designs/../x.toml", b"x"), Err(Error::Damaged { .. })));
    assert!(matches!(with("designs/bad name.toml", b"x"), Err(Error::Damaged { .. })));
    assert!(matches!(with("designs/big.toml", &vec![b'#'; limits::MAX_DESIGN as usize + 1]), Err(Error::TooLarge { .. })));
    assert!(matches!(with("assets/../../etc", b"x"), Err(Error::Damaged { .. })));
    assert!(matches!(with("assets/.hidden", b"x"), Err(Error::Damaged { .. })));
    assert!(matches!(with("builds/x.json", &[0xff]), Err(Error::Damaged { .. })));
    let mut no_toml = good.clone();
    no_toml.remove("book.toml");
    assert_eq!(Files::from_map(&no_toml).unwrap_err(), Error::NotABook);
    let mut many = good.clone();
    for i in 0..=limits::MAX_BUILDS {
        many.insert(format!("builds/e{i}.json"), b"{}".to_vec());
    }
    assert!(matches!(Files::from_map(&many), Err(Error::TooLarge { .. })));
    // A folder without book.toml is not a project.
    assert_eq!(Folder::open(&tmp("empty")).unwrap_err(), Error::NotABook);
}

/// Every example in `docs/format.md` parses as the file it is an example of.
#[test]
fn every_example_in_the_format_doc_parses() {
    let doc = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/format.md")).unwrap();
    let mut seen = 0;
    for part in doc.split("<!-- example: ").skip(1) {
        let (what, rest) = part.split_once(" -->").unwrap();
        let body = rest.split("```").nth(1).unwrap();
        let body = body.split_once('\n').unwrap().1;
        if what.starts_with("book.toml") {
            let p = toml_file::read(body).unwrap_or_else(|e| panic!("{what}: {e}"));
            assert_eq!(p.schema, 1);
        } else if what.ends_with(".json") {
            serde_json::from_str::<serde_json::Value>(body).unwrap_or_else(|e| panic!("{what}: {e}"));
        }
        seen += 1;
    }
    assert!(seen >= 3, "the format doc has its examples");
}

fn text() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 .,'’—“”é-]{0,24}"
}

fn one_line() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 .,'’é]{1,24}"
}

prop_compose! {
    fn edition()(id in "[a-z][a-z0-9-]{0,10}", name in text(), isbn in text(), trim in "[0-9x.]{0,6}", print in any::<bool>(), embed in any::<bool>(), notes in 0..3u8, price in text()) -> Edition {
        Edition {
            id,
            name,
            format: if print { Format::Paperback } else { Format::Ebook },
            isbn,
            trim,
            paper: print.then_some(Paper::Cream),
            embed_fonts: embed,
            notes: [NoteStyle::Popup, NoteStyle::ChapterEnd, NoteStyle::BookEnd][notes as usize],
            price_notes: price,
            ..Edition::default()
        }
    }
}

prop_compose! {
    fn project()(
        date in "[0-9]{4}-[0-9]{2}-[0-9]{2}",
        linked in any::<bool>(),
        editions in proptest::collection::vec(edition(), 0..4),
        elements in proptest::collection::vec((text(), any::<bool>()), 0..4),
        sections in proptest::collection::btree_map(0u64..50, (text(), text(), any::<bool>()), 0..5),
        fixes in proptest::collection::vec((one_line(), text(), text()), 0..4),
        contributors in proptest::collection::vec(text(), 0..3),
    ) -> BookProject {
        let mut p = BookProject::new(if linked { Source::Manuscript("../A Novel.wsm".into()) } else { Source::default() }, &date);
        let mut seen = std::collections::BTreeSet::new();
        p.editions = editions.into_iter().filter(|e| seen.insert(e.id.clone())).collect();
        p.elements = elements.into_iter().enumerate().map(|(i, (t, back))| Element {
            id: format!("el-{i}"),
            kind: Some(if back { ElementKind::AboutTheAuthor } else { ElementKind::Dedication }),
            placement: if back { Placement::Back } else { Placement::Front },
            text: t,
            ..Element::default()
        }).collect();
        p.sections = sections.into_iter().map(|(n, (sub, epi, print))| (SectionId::from_counter(n), SectionSettings {
            subtitle: sub, epigraph: epi, only: if print { Only::Print } else { Only::Both }, ..SectionSettings::default()
        })).collect();
        p.fixes = fixes.into_iter().enumerate().map(|(i, (find, replace, note))| Fix {
            id: format!("fix-{i}"), section: Some(SectionId::from_counter(i as u64)), find, replace, note, ..Fix::default()
        }).collect();
        p.metadata.contributors = contributors.into_iter().map(|name| Contributor { name, role: Role::Editor }).collect();
        p
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn random_projects_round_trip_exactly(p in project()) {
        let text = toml_file::write(&p).unwrap();
        let back = toml_file::read(&text).unwrap();
        prop_assert_eq!(&back, &p);
        prop_assert_eq!(toml_file::write(&back).unwrap(), text);
    }
}

/// A release registers both project types from the data in `xtask/src/main.rs`
/// (`cargo xtask package` writes the menu entry, the MIME file, the property list and the
/// Windows scripts from it). The packed type's media type there is the one this crate writes
/// first into every packed project; the xtask cannot read this crate, so the test reads its data.
#[test]
fn the_release_data_registers_both_project_types() {
    let data = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../xtask/src/main.rs")).unwrap();
    let packed = format!("mime: \"{}\"", wordsmith_book_format::pack::MIMETYPE);
    assert!(data.contains(&packed), "the packed project's media type in the release data is not {}", wordsmith_book_format::pack::MIMETYPE);
    assert!(data.contains("mime: \"application/x-wordsmith-book\""), "the project folder's media type");
    assert!(data.contains("extensions: &[\"wsbk\"]") && data.contains("extensions: &[\"wsbkz\"]"), "both extensions");
}
