// SPDX-License-Identifier: MIT OR Apache-2.0
//! What each way of writing a Book project puts on disk, as the SHA-256 of every file
//! (`tests/on_disk.rs` compares it with `tests/golden/project-files.txt`). Every name here is
//! one Wordsmith Book 0.9.0's format crate has, under the same signature: this file was not
//! changed when the folder layer became the core's, so the reference it wrote then still judges.

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;
use std::time::{Duration, SystemTime};

use sha2::{Digest, Sha256};
use wordsmith_book_format::{Files, Folder, pack, source};
use wordsmith_book_model::{BookProject, Source};
use wordsmith_manuscript_model::{Area, Item, ItemId, ItemText, Kind, Place, Project, Role, Text};

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// Every file under `root`, by its path inside it, with `/` between the parts.
fn tree(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let rel = path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
                out.insert(rel, std::fs::read(&path).unwrap());
            }
        }
    }
    out
}

fn section(out: &mut String, title: &str, files: &BTreeMap<String, Vec<u8>>) {
    let _ = writeln!(out, "[{title}]");
    for (path, bytes) in files {
        let _ = writeln!(out, "{path} {}", sha(bytes));
    }
    out.push('\n');
}

fn one(out: &mut String, title: &str, name: &str, bytes: &[u8]) {
    let _ = writeln!(out, "[{title}]\n{name} {}\n", sha(bytes));
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            copy(&p, &to.join(e.file_name()));
        } else {
            std::fs::copy(&p, to.join(e.file_name())).unwrap();
        }
    }
}

/// A small Manuscript project written for the test: a title page and a chapter of two scenes.
fn manuscript() -> Project {
    let id = ItemId::from_counter;
    let mut p = Project::new();
    p.settings.book.title = "The Lighthouse".into();
    p.settings.book.author = "A. Writer".into();
    p.binder.insert(Place::end_of(Area::Manuscript), Item::with_id(id(1), Kind::Text, "Title Page").role(Role::FrontMatter)).unwrap();
    p.binder.insert(Place::end_of(Area::Manuscript), Item::with_id(id(2), Kind::Folder, "Arrival").role(Role::Chapter)).unwrap();
    for (n, title) in [(3, "Dusk"), (4, "Night")] {
        p.binder.insert(Place::inside(Area::Manuscript, id(2)), Item::with_id(id(n), Kind::Text, title).role(Role::Scene)).unwrap();
    }
    p.texts.insert(id(1), ItemText::with_body(Text::from_paragraphs(&["The Lighthouse", "A. Writer"])));
    p.texts.insert(id(3), ItemText::with_body(Text::from_paragraphs(&["The keeper climbed the rock.", "The lamp was cold."])));
    p.texts.insert(id(4), ItemText::with_body(Text::from_paragraphs(&["The sea was quiet."])));
    p
}

/// A book on an owned source with every kind of file Book keeps: a design, assets (pictures
/// stored as they are, a font compressed, one in a folder), a build record.
fn book() -> Files {
    let mut f = Files::new(BookProject::new(Source::Owned("source.wsm".into()), "2026-11-24"));
    f.designs.insert("mine".into(), "schema = 1\nid = \"mine\"\nname = \"Mine\"\n".into());
    f.assets.insert("cover.png".into(), vec![0x89, b'P', b'N', b'G', 1, 2, 3]);
    f.assets.insert("pictures/map.jpg".into(), vec![0xFF, 0xD8, 0xFF, 4, 5, 6]);
    f.assets.insert("fonts/display.otf".into(), b"OTTO a font written for the test, repeated, repeated, repeated".to_vec());
    f.builds.insert("paperback".into(), "{\"pages\": 12}".into());
    f
}

/// The same book after a session of work: the date and the design changed, a build added, a
/// picture removed.
fn worked(mut f: Files) -> Files {
    f.project.date = "2026-11-26".into();
    f.designs.insert("mine".into(), "schema = 1\nid = \"mine\"\nname = \"Mine, again\"\n".into());
    f.builds.insert("ebook".into(), "{\"chapters\": 1}".into());
    f.assets.remove("pictures/map.jpg");
    f
}

/// What each way of writing a project puts on disk, under the folder `base`.
pub fn written(base: &Path) -> String {
    let _ = std::fs::remove_dir_all(base);
    let mut out = String::new();

    // A new project folder with its owned source, and a file of the user's own beside them.
    let root = base.join("The Lighthouse.wsbk");
    let first = book();
    let mut folder = Folder::create(&root, &first).unwrap();
    source::write_owned(&root, "source.wsm", &manuscript()).unwrap();
    std::fs::write(root.join("notes.txt"), "The user's own notes.\n").unwrap();
    section(&mut out, "a new project folder", &tree(&root));

    // Saved again after a session of work.
    let second = worked(first);
    folder.save(&second).unwrap();
    section(&mut out, "the folder after a save", &tree(&root));

    // book.toml edited by hand, opened and saved: Book writes it in its own form.
    let toml = std::fs::read_to_string(root.join("book.toml")).unwrap();
    std::fs::write(root.join("book.toml"), format!("{toml}\n# A line added by hand.\n")).unwrap();
    let (mut reopened, read) = Folder::open(&root).unwrap();
    reopened.save(&read).unwrap();
    section(&mut out, "the folder after a save of book.toml edited by hand", &tree(&root));

    // The packed project, from the folder (with the owned source) and from memory.
    let packed = pack::pack_folder(&root, &second).unwrap();
    one(&mut out, "the folder packed", "The Lighthouse.wsbkz", &packed);
    one(
        &mut out,
        "the project packed from memory",
        "The Lighthouse.wsbkz",
        &pack::pack(&pack::Packed { files: second.clone(), ..pack::Packed::default() }).unwrap(),
    );

    // The recovery copy of unsaved work, then a save with recovery around it.
    let mut third = second.clone();
    third.project.date = "2026-11-27".into();
    pack::write_recovery(&root, &third).unwrap();
    let recovery = pack::recovery_path(&root);
    let name = recovery.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/");
    one(&mut out, "the recovery copy", &name, &std::fs::read(&recovery).unwrap());
    pack::save_safely(&mut reopened, &third).unwrap();
    section(&mut out, "the folder after a save with recovery", &tree(&root));

    // Two backups in the same second, at a fixed time.
    let store = base.join("The Lighthouse Backups");
    let at = SystemTime::UNIX_EPOCH + Duration::from_secs(1_795_000_000);
    for _ in 0..2 {
        let made = pack::backup(&root, &third, &store, "The Lighthouse", 3, at).unwrap();
        let name = made.file_name().unwrap().to_string_lossy().into_owned();
        one(&mut out, "a backup", &name, &std::fs::read(&made).unwrap());
    }

    // A packed project unpacked into a folder.
    let dest = base.join("Unpacked.wsbk");
    pack::unpack_into(&packed, &dest).unwrap();
    section(&mut out, "a packed project unpacked", &tree(&dest));

    // A book on a linked Manuscript project: packed, it carries a packed copy of the source.
    let linked = base.join("Linked");
    wordsmith_manuscript_format::folder::Folder::create(&linked.join("The Lighthouse.wsm"), &manuscript()).unwrap();
    let on_linked = Files::new(BookProject::new(Source::Manuscript("../The Lighthouse.wsm".into()), "2026-11-24"));
    Folder::create(&linked.join("Book.wsbk"), &on_linked).unwrap();
    let packed = pack::pack_folder(&linked.join("Book.wsbk"), &on_linked).unwrap();
    one(&mut out, "a book on a linked source packed", "Book.wsbkz", &packed);
    let away = base.join("Away").join("Book.wsbk");
    pack::unpack_into(&packed, &away).unwrap();
    section(&mut out, "a book on a linked source unpacked elsewhere", &tree(&away));

    // A project of schema 0, opened and saved: it is written as schema 1.
    let old = base.join("Old.wsbk");
    copy(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/projects/version-0.wsbk"), &old);
    let (mut folder, read) = Folder::open(&old).unwrap();
    folder.save(&read).unwrap();
    section(&mut out, "a schema 0 project saved", &tree(&old));
    out
}
