// SPDX-License-Identifier: MIT OR Apache-2.0
//! Sources (plan M1.2): the Manuscript project fixture through the book view (every word, role
//! and id), linked folders and packed projects, the owned source written and read back, a linked
//! source changed on disk, other book view versions refused, and the Manuscript folder's bytes
//! unchanged after every test.

#[path = "../examples/manuscript_fixture/gulliver.rs"]
mod gulliver;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use wordsmith_book_format::source::{self, Change, Kind};
use wordsmith_book_model::Source;
use wordsmith_manuscript_format::book::{self, FrontKind};
use wordsmith_manuscript_model::{Block, Inline, Role};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture() -> PathBuf {
    root().join("tests/fixtures/novels/gullivers-travels.wsm")
}

fn fixture_text() -> String {
    std::fs::read_to_string(root().join("tests/fixtures/novels/gullivers-travels.txt")).unwrap()
}

fn tmp(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("sources").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Every file of a folder, by path, with its bytes.
fn tree(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.insert(p.strip_prefix(dir).unwrap().display().to_string(), std::fs::read(&p).unwrap());
            }
        }
    }
    out
}

fn copy(from: &Path, to: &Path) {
    for (rel, bytes) in tree(from) {
        let p = to.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
}

/// A Book project folder in `dir`; its source is a sibling, by a relative path.
fn linking(dir: &Path) -> PathBuf {
    let p = dir.join("Book.wsbk");
    std::fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn the_fixture_through_the_book_view_gives_every_word_role_and_id() {
    let before = tree(&fixture());
    let dir = tmp("view");
    copy(&fixture(), &dir.join("Gulliver.wsm"));
    let project_root = linking(&dir);
    let opened = source::open(&project_root, &Source::Manuscript("../Gulliver.wsm".into())).unwrap();
    assert_eq!(opened.kind, Kind::LinkedFolder);
    let b = &opened.book;
    assert_eq!(b.version, 1);
    let (project, ids) = gulliver::project(&fixture_text());
    // The view from disk is the view of the project in memory, exactly.
    assert_eq!(*b, book::from_project(&project));
    // Roles and ids.
    assert_eq!(b.front.len(), 3);
    assert_eq!(b.front[0].id, ids.title_page);
    assert_eq!(b.front[0].kind, FrontKind::TitlePage);
    assert_eq!(b.front[1..].iter().map(|s| s.id).collect::<Vec<_>>(), ids.front);
    assert_eq!(
        b.body.iter().map(|s| (s.id, s.role, s.number)).collect::<Vec<_>>(),
        ids.parts.iter().enumerate().map(|(i, id)| (*id, Role::Part, Some(i as u32 + 1))).collect::<Vec<_>>()
    );
    let chapters: Vec<_> = b.body.iter().flat_map(|p| p.children.iter()).collect();
    assert_eq!(chapters.iter().map(|c| c.id).collect::<Vec<_>>(), ids.chapters);
    assert!(chapters.iter().enumerate().all(|(i, c)| c.role == Role::Chapter && c.number == Some(i as u32 + 1)));
    assert_eq!(b.back.len(), 1);
    assert_eq!(b.back[0].kind, FrontKind::AboutTheAuthor);
    // Left out: the item excluded from export and the research note.
    let all: Vec<_> = book::outline(b).into_iter().map(|(id, _)| id).collect();
    assert!(!all.contains(&ids.cut) && !all.contains(&ids.research));
    // Every word of the fixture's chapters, in order, from the text the fixture was made from.
    let parsed = gulliver::parse(&fixture_text());
    let mut want: Vec<String> = Vec::new();
    for (head, pars) in &parsed.front {
        let _ = head;
        for par in pars {
            let text = par.iter().map(|l| l.trim()).collect::<Vec<_>>().join(" ");
            want.extend(wordsmith_manuscript_model::words::words(&text.replace('_', "")));
        }
    }
    for (head, chapters) in &parsed.parts {
        want.extend(wordsmith_manuscript_model::words::words(&gulliver::part_title(head)));
        for (_, pars) in chapters {
            for par in pars {
                let text = par.iter().map(|l| l.trim()).collect::<Vec<_>>().join(" ");
                let without_refs: String =
                    parsed.notes.iter().fold(text, |t, (l, _)| t.replace(&format!(" [{l}]"), "").replace(&format!("[{l}]"), ""));
                want.extend(wordsmith_manuscript_model::words::words(&without_refs.replace('_', "")));
            }
        }
    }
    let got: Vec<String> = b.front[1..]
        .iter()
        .chain(&b.body)
        .flat_map(|s| {
            let mut w = Vec::new();
            let mut stack = vec![s];
            while let Some(s) = stack.pop() {
                if matches!(s.role, Role::Part | Role::Chapter) {
                    w.extend(wordsmith_manuscript_model::words::words(&s.title));
                }
                let t = wordsmith_manuscript_model::Text { blocks: s.content.clone(), notes: Vec::new() };
                w.extend(t.plain_paragraphs().iter().flat_map(|p| wordsmith_manuscript_model::words::words(p)));
                stack.extend(s.children.iter().rev());
            }
            w
        })
        .collect();
    assert_eq!(got.len(), want.len());
    if let Some(at) = got.iter().zip(&want).position(|(a, b)| a != b) {
        panic!(
            "words differ at {at}: view {:?}, text {:?}",
            &got[at.saturating_sub(3)..(at + 4).min(got.len())],
            &want[at.saturating_sub(3)..(at + 4).min(want.len())]
        );
    }
    // Notes stay notes, at their references.
    let with_note = chapters.iter().find(|c| !c.notes.is_empty()).unwrap();
    assert!(with_note.content.iter().any(|blk| matches!(blk, Block::Para(c) if c.iter().any(|i| matches!(i, Inline::NoteRef { .. })))));
    // What the hand-off review records: the view keeps comment anchors on the text (the comments
    // themselves are not in it); Book ignores them.
    let first = &chapters[0].content[0];
    assert!(matches!(first, Block::Para(c) if matches!(c.first(), Some(Inline::Span { marks, .. }) if marks.comment.is_some())));
    // Nothing was written into the Manuscript folder.
    assert_eq!(tree(&fixture()), before);
    assert_eq!(tree(&dir.join("Gulliver.wsm")), before);
}

#[test]
fn a_packed_project_gives_the_same_book() {
    let before = tree(&fixture());
    let dir = tmp("packed");
    let bytes = wordsmith_manuscript_format::pack::pack_folder(&fixture()).unwrap();
    std::fs::write(dir.join("Gulliver.wsmz"), &bytes).unwrap();
    let project_root = linking(&dir);
    let opened = source::open(&project_root, &Source::Manuscript("../Gulliver.wsmz".into())).unwrap();
    assert_eq!(opened.kind, Kind::LinkedPacked);
    let (project, _) = gulliver::project(&fixture_text());
    assert_eq!(opened.book, book::from_project(&project));
    assert!(!opened.changed_on_disk().unwrap());
    assert_eq!(std::fs::read(dir.join("Gulliver.wsmz")).unwrap(), bytes);
    assert_eq!(tree(&fixture()), before);
}

#[test]
fn an_owned_source_is_written_and_read_back_identical() {
    let dir = tmp("owned").join("Book.wsbk");
    std::fs::create_dir_all(&dir).unwrap();
    let (project, _) = gulliver::project(&fixture_text());
    source::write_owned(&dir, "source.wsm", &project).unwrap();
    let opened = source::open(&dir, &Source::Owned("source.wsm".into())).unwrap();
    assert_eq!(opened.kind, Kind::Owned);
    assert_eq!(opened.book, book::from_project(&project));
    // Saving again changes nothing; the owned source also opens in Manuscript's own reader.
    let before = tree(&dir.join("source.wsm"));
    source::write_owned(&dir, "source.wsm", &project).unwrap();
    assert_eq!(tree(&dir.join("source.wsm")), before);
    assert!(wordsmith_manuscript_format::folder::Folder::open(&dir.join("source.wsm")).is_ok());
    // A missing owned source is a clear error.
    assert!(source::open(&dir, &Source::Owned("other.wsm".into())).is_err());
}

#[test]
fn a_linked_source_changed_on_disk_is_noticed_and_read_again() {
    let before = tree(&fixture());
    let dir = tmp("changed");
    let ms = dir.join("Gulliver.wsm");
    copy(&fixture(), &ms);
    let project_root = linking(&dir);
    let src = Source::Manuscript("../Gulliver.wsm".into());
    let opened = source::open(&project_root, &src).unwrap();
    let sums = source::checksums(&opened.book);
    assert!(!opened.changed_on_disk().unwrap());
    // The writer changes chapter three in Manuscript.
    let (_, ids) = gulliver::project(&fixture_text());
    let (mut folder, mut project) = wordsmith_manuscript_format::folder::Folder::open(&ms).unwrap();
    let t = project.texts.get_mut(&ids.chapters[2]).unwrap();
    t.body.blocks.push(Block::Para(vec![Inline::plain("A sentence the writer added.")]));
    folder.save(&project).unwrap();
    assert!(opened.changed_on_disk().unwrap());
    let again = source::open(&project_root, &src).unwrap();
    let changes = source::changed_sections(&sums, &source::checksums(&again.book));
    assert_eq!(changes, vec![(ids.chapters[2], Change::Changed)]);
    // Book wrote nothing into either folder.
    assert_eq!(tree(&fixture()), before);
}

#[test]
fn other_book_view_versions_are_refused_clearly() {
    let (project, _) = gulliver::project(&fixture_text());
    let mut b = book::from_project(&project);
    assert!(source::check_version(&b).is_ok());
    b.version = 2;
    let e = source::check_version(&b).unwrap_err();
    assert_eq!(e, wordsmith_book_format::Error::SourceVersion(wordsmith_book_format::Outside::BookView(2)));
    assert!(e.to_string().contains("book view version 2") && e.to_string().contains("reads book view version 1"), "{e}");
    // A path that is not a Manuscript project is refused before anything is read.
    assert!(source::open(Path::new("."), &Source::Manuscript("../notes.txt".into())).is_err());
}

/// The fixture as a project of format `schema`: a copy with that number in its `project.toml`.
fn fixture_in_format(dir: &Path, schema: u32) -> PathBuf {
    let ms = dir.join("My Novel.wsm");
    copy(&fixture(), &ms);
    let toml = std::fs::read_to_string(ms.join("project.toml")).unwrap();
    let now = format!("schema = {}\n", wordsmith_manuscript_format::SCHEMA);
    assert!(toml.starts_with(&now), "the fixture is in the newest format Book reads");
    std::fs::write(ms.join("project.toml"), toml.replacen(&now, &format!("schema = {schema}\n"), 1)).unwrap();
    ms
}

#[test]
fn a_manuscript_project_in_a_format_book_reads_opens_as_a_folder_and_packed() {
    let newest = *wordsmith_book_format::MANUSCRIPT_SCHEMAS.end();
    assert!(source::check_schema(newest).is_ok() && source::check_schema(*wordsmith_book_format::MANUSCRIPT_SCHEMAS.start()).is_ok());
    let dir = tmp("format-inside");
    let ms = fixture_in_format(&dir, newest);
    let project_root = dir.join("Book.wsbk");
    std::fs::create_dir_all(&project_root).unwrap();
    let folder = source::open(&project_root, &Source::Manuscript("../My Novel.wsm".into())).unwrap();
    assert_eq!(folder.kind, Kind::LinkedFolder);
    std::fs::write(dir.join("My Novel.wsmz"), wordsmith_manuscript_format::pack::pack_folder(&ms).unwrap()).unwrap();
    let packed = source::open(&project_root, &Source::Manuscript("../My Novel.wsmz".into())).unwrap();
    assert_eq!(packed.kind, Kind::LinkedPacked);
    assert_eq!(source::checksums(&packed.book), source::checksums(&folder.book), "the same words in every section");
}

#[test]
fn a_manuscript_project_in_a_format_outside_the_range_is_refused_in_books_words() {
    use wordsmith_book_format::{Error, Outside};
    let newer = *wordsmith_book_format::MANUSCRIPT_SCHEMAS.end() + 1;
    assert_eq!(source::check_schema(newer), Err(Error::SourceVersion(Outside::Schema(newer))));
    let dir = tmp("format-outside");
    let ms = fixture_in_format(&dir, newer);
    let before = tree(&ms);
    let project_root = dir.join("Book.wsbk");
    std::fs::create_dir_all(&project_root).unwrap();
    // As a folder, and as the packed project a newer Manuscript would write.
    std::fs::write(dir.join("My Novel.wsmz"), wordsmith_manuscript_format::pack::pack_folder(&ms).unwrap()).unwrap();
    for link in ["../My Novel.wsm", "../My Novel.wsmz"] {
        let Err(e) = source::open(&project_root, &Source::Manuscript(link.into())) else { panic!("{link} opened") };
        assert_eq!(e, Error::SourceVersion(Outside::Schema(newer)), "{link}");
        let said = e.to_string();
        // Both versions by name, and Book's advice, not Manuscript's ("Update the app").
        assert!(said.contains(&format!("is in format {newer}")), "{said}");
        assert!(said.contains(&format!("reads Manuscript formats 1 to {}", newer - 1)), "{said}");
        assert!(
            said.contains(concat!("Wordsmith Book ", env!("CARGO_PKG_VERSION"))) && said.ends_with("Update Wordsmith Book to open it."),
            "{said}"
        );
        assert!(!said.contains("Update the app"), "{said}");
    }
    // An owned source a newer Manuscript saved is refused the same way, and is not written over.
    let own = dir.join("Own.wsbk");
    copy(&ms, &own.join(wordsmith_book_model::OWNED_SOURCE));
    let (project, _) = gulliver::project(&fixture_text());
    assert_eq!(source::write_owned(&own, wordsmith_book_model::OWNED_SOURCE, &project), Err(Error::SourceVersion(Outside::Schema(newer))));
    assert_eq!(tree(&own.join(wordsmith_book_model::OWNED_SOURCE)), before);
    assert_eq!(tree(&ms), before, "Book wrote nothing into the project");
}

#[test]
fn checksums_cover_every_section_and_change_with_words() {
    let (project, ids) = gulliver::project(&fixture_text());
    let b = book::from_project(&project);
    let sums = source::checksums(&b);
    assert_eq!(sums.len(), book::outline(&b).len());
    let mut changed = project.clone();
    changed.texts.get_mut(&ids.about).unwrap().body = wordsmith_manuscript_model::Text::from_paragraphs(&["A new biography."]);
    let new = source::checksums(&book::from_project(&changed));
    assert_eq!(source::changed_sections(&sums, &new), vec![(ids.about, Change::Changed)]);
    let mut fewer = new.clone();
    fewer.remove(&ids.about);
    assert_eq!(source::changed_sections(&new, &fewer), vec![(ids.about, Change::Removed)]);
    assert_eq!(source::changed_sections(&fewer, &new), vec![(ids.about, Change::Added)]);
}

#[test]
fn a_sources_pictures_are_read_by_path_and_only_under_media() {
    use wordsmith_book_format::source::picture_path_ok;
    assert!(picture_path_ok("media/0001.png"));
    for bad in ["../x.png", "media/../../etc", "/media/x.png", "text/x.md", "media\\x.png", "C:media/x"] {
        assert!(!picture_path_ok(bad), "{bad}");
    }
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("pictures");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let novels = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels");
    let src = dir.join("book.wsm");
    let (_, project) = wordsmith_manuscript_format::folder::Folder::open(&novels.join("gullivers-travels.wsm")).unwrap();
    wordsmith_manuscript_format::folder::Folder::create(&src, &project).unwrap();
    std::fs::create_dir_all(src.join("media")).unwrap();
    std::fs::write(src.join("media/map.png"), b"picture bytes").unwrap();
    let opened = wordsmith_book_format::source::open(&dir, &wordsmith_book_model::Source::Manuscript("book.wsm".into())).unwrap();
    assert_eq!(opened.picture("media/map.png").as_deref(), Some(&b"picture bytes"[..]));
    assert!(opened.picture("media/none.png").is_none());
    assert!(opened.picture("../book.wsm/media/map.png").is_none());
}
