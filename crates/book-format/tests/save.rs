// SPDX-License-Identifier: MIT OR Apache-2.0
//! Saving (plan M1.3): crash injection after every file write, packing twice to the same bytes,
//! damaged and hostile packed projects, recovery and backups.

#[path = "../examples/manuscript_fixture/gulliver.rs"]
mod gulliver;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use wordsmith_book_format::folder::{Disk, Sink};
use wordsmith_book_format::{Error, Files, Folder, pack, source};
use wordsmith_book_model::{BookProject, Source};

fn tmp(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("save").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn before() -> Files {
    let mut f = Files::new(BookProject::new(Source::default(), "2026-11-24"));
    f.designs.insert("mine".into(), "schema = 1\n".into());
    f.designs.insert("old".into(), "schema = 1\nname = \"Old\"\n".into());
    f.builds.insert("paperback".into(), "{}".into());
    f
}

fn after() -> Files {
    let mut f = before();
    f.project.date = "2026-12-24".into();
    f.designs.insert("mine".into(), "schema = 1\nname = \"Changed\"\n".into());
    f.designs.remove("old");
    f.assets.insert("cover.png".into(), vec![1, 2, 3]);
    f.builds.clear();
    f
}

/// A sink that writes to disk and then stops, as a crash would, after `left` writes.
struct Crash<'a> {
    disk: Disk<'a>,
    left: usize,
}

impl Sink for Crash<'_> {
    fn write(&mut self, rel: &str, bytes: &[u8]) -> Result<(), Error> {
        if self.left == 0 {
            return Err(Error::Io { file: rel.into(), why: "the machine stopped".into() });
        }
        self.left -= 1;
        self.disk.write(rel, bytes)
    }

    fn remove(&mut self, rel: &str) -> Result<(), Error> {
        if self.left == 0 {
            return Err(Error::Io { file: rel.into(), why: "the machine stopped".into() });
        }
        self.left -= 1;
        self.disk.remove(rel)
    }
}

#[test]
fn every_crash_reopens_to_the_last_good_save_or_the_recovered_work() {
    // How many steps a whole save takes.
    let dir = tmp("crash-count").join("A.wsbk");
    let mut folder = Folder::create(&dir, &before()).unwrap();
    let whole = folder.save(&after()).unwrap();
    let steps = whole.written.len() + whole.removed.len();
    assert!(steps >= 4, "{whole:?}");
    for n in 0..=steps {
        let dir = tmp(&format!("crash-{n}")).join("A.wsbk");
        let mut folder = Folder::create(&dir, &before()).unwrap();
        pack::write_recovery(&dir, &after()).unwrap();
        let result = folder.save_to(&after(), &mut Crash { disk: Disk(&dir), left: n });
        // The machine comes back: the project opens (every file whole), and the work is either
        // saved or recoverable.
        let (_, opened) = Folder::open(&dir).unwrap_or_else(|e| panic!("after {n} writes: {e}"));
        if result.is_ok() {
            assert_eq!(opened, after());
        } else {
            assert_eq!(pack::recovered(&dir), Some(after()), "after {n} writes the work is recoverable");
            assert!(opened.project == before().project || opened.project == after().project);
        }
    }
    // A save that finishes removes the recovery file.
    let dir = tmp("crash-safe").join("A.wsbk");
    let mut folder = Folder::create(&dir, &before()).unwrap();
    pack::save_safely(&mut folder, &after()).unwrap();
    assert_eq!(pack::recovered(&dir), None);
    assert_eq!(Folder::open(&dir).unwrap().1, after());
}

#[test]
fn packing_twice_gives_the_same_bytes_and_unpacks_to_the_same_project() {
    let dir = tmp("pack").join("Gulliver.wsbk");
    let text = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels/gullivers-travels.txt")).unwrap();
    let (project, _) = gulliver::project(&text);
    let files = after();
    Folder::create(&dir, &files).unwrap();
    source::write_owned(&dir, "source.wsm", &project).unwrap();
    let a = pack::pack_folder(&dir, &files).unwrap();
    let b = pack::pack_folder(&dir, &files).unwrap();
    assert!(a == b, "packing twice gives the same bytes");
    assert_eq!(&a[30..38], b"mimetype");
    let p = pack::unpack(&a).unwrap();
    assert_eq!(p.files, files);
    assert!(p.owned.contains_key("project.toml") && p.owned.contains_key("binder.toml"));
    // Opened as a new folder, it is the same project with the same source.
    let dest = tmp("pack-open").join("Copy.wsbk");
    let (_, back) = pack::unpack_into(&a, &dest).unwrap();
    assert_eq!(back, files);
    let src = source::open(&dest, &Source::Owned("source.wsm".into())).unwrap();
    assert_eq!(src.book, wordsmith_manuscript_format::book::from_project(&project));
    // Never over a folder that holds something.
    assert!(pack::unpack_into(&a, &dest).is_err());
}

#[test]
fn a_linked_source_travels_as_a_packed_copy() {
    let dir = tmp("linked");
    let ms = dir.join("Gulliver.wsm");
    let text = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels/gullivers-travels.txt")).unwrap();
    let (project, _) = gulliver::project(&text);
    wordsmith_manuscript_format::folder::Folder::create(&ms, &project).unwrap();
    let book = dir.join("Book.wsbk");
    let files = Files::new(BookProject::new(Source::Manuscript("../Gulliver.wsm".into()), "2026-11-24"));
    Folder::create(&book, &files).unwrap();
    let bytes = pack::pack_folder(&book, &files).unwrap();
    assert!(pack::unpack(&bytes).unwrap().linked.is_some());
    // Somewhere the linked folder is not: the copy is read.
    let away = tmp("linked-away").join("far").join("Book.wsbk");
    pack::unpack_into(&bytes, &away).unwrap();
    let src = source::open(&away, &Source::Manuscript("../Gulliver.wsm".into())).unwrap();
    assert_eq!(src.kind, source::Kind::LinkedPacked);
    assert_eq!(src.book, wordsmith_manuscript_format::book::from_project(&project));
}

fn zip_of(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, bytes) in entries {
        w.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
        w.write_all(bytes).unwrap();
    }
    w.finish().unwrap().into_inner()
}

#[test]
fn damaged_and_hostile_packed_projects_are_refused_cleanly() {
    let good = pack::pack(&pack::Packed { files: after(), ..pack::Packed::default() }).unwrap();
    let toml = wordsmith_book_format::toml_file::write(&after().project).unwrap();
    let m = pack::MIMETYPE.as_bytes();
    for (what, bytes) in [
        ("empty", Vec::new()),
        ("not a zip", b"PK\x03\x04 this is not a zip at all".to_vec()),
        ("cut short", good[..good.len() / 2].to_vec()),
        ("no mimetype first", zip_of(&[("book.toml", toml.as_bytes()), ("mimetype", m)])),
        ("another app's mimetype", zip_of(&[("mimetype", b"application/epub+zip"), ("book.toml", toml.as_bytes())])),
        ("a name above the project", zip_of(&[("mimetype", m), ("book.toml", toml.as_bytes()), ("../escape.txt", b"x")])),
        ("an absolute name", zip_of(&[("mimetype", m), ("book.toml", toml.as_bytes()), ("/etc/x", b"x")])),
        ("a hidden name", zip_of(&[("mimetype", m), ("book.toml", toml.as_bytes()), ("assets/.x", b"x")])),
        ("no book.toml", zip_of(&[("mimetype", m)])),
        ("a damaged book.toml", zip_of(&[("mimetype", m), ("book.toml", b"schema = [[[")])),
        ("a newer project", zip_of(&[("mimetype", m), ("book.toml", toml.replace("schema = 1", "schema = 9").as_bytes())])),
    ] {
        assert!(pack::unpack(&bytes).is_err(), "{what} is refused");
    }
    assert!(pack::unpack(&good).is_ok());
}

#[test]
fn backups_keep_the_newest_and_never_overwrite_one() {
    let root = tmp("backup-project").join("A.wsbk");
    Folder::create(&root, &after()).unwrap();
    let dir = tmp("backups");
    let t0 = SystemTime::UNIX_EPOCH + Duration::from_secs(1_790_000_000);
    let a = pack::backup(&root, &after(), &dir, "A", 2, t0).unwrap();
    let b = pack::backup(&root, &after(), &dir, "A", 2, t0).unwrap();
    assert_ne!(a, b, "a second backup in the same second gets a number");
    let c = pack::backup(&root, &after(), &dir, "A", 2, t0 + Duration::from_secs(60)).unwrap();
    let all = pack::backups(&dir, "A");
    assert_eq!(all.len(), 2);
    assert!(all.contains(&c));
    assert!(a.file_name().unwrap().to_string_lossy().ends_with(" UTC.wsbkz"));
    assert_eq!(pack::unpack(&std::fs::read(&c).unwrap()).unwrap().files, after());
}

#[test]
fn recovery_keeps_unsaved_work_until_it_is_discarded() {
    let root = tmp("recovery").join("A.wsbk");
    Folder::create(&root, &before()).unwrap();
    assert_eq!(pack::recovered(&root), None);
    pack::write_recovery(&root, &after()).unwrap();
    assert_eq!(pack::recovered(&root), Some(after()));
    // The recovery folder is not part of the project.
    assert_eq!(Folder::open(&root).unwrap().1, before());
    pack::discard_recovery(&root).unwrap();
    assert_eq!(pack::recovered(&root), None);
    // A damaged recovery file is ignored, not trusted.
    std::fs::write(pack::recovery_path(&root), b"damaged").unwrap();
    assert_eq!(pack::recovered(&root), None);
}
