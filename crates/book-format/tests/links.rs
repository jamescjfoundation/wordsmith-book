// SPDX-License-Identifier: MIT OR Apache-2.0
//! Symbolic links in a project folder are never followed (core decision 0018): a project cannot
//! name a file outside itself. Unix only: the links are made with the system's call there.

#![cfg(unix)]

use std::path::{Path, PathBuf};

use wordsmith_book_format::{Error, Files, Folder, pack};
use wordsmith_book_model::{BookProject, Source};

fn tmp(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("links").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn files() -> Files {
    let mut f = Files::new(BookProject::new(Source::default(), "2026-11-27"));
    f.assets.insert("cover.png".into(), vec![1, 2, 3]);
    f
}

#[test]
fn a_linked_asset_is_not_read_packed_or_written_over() {
    let dir = tmp("asset");
    let root = dir.join("A.wsbk");
    Folder::create(&root, &files()).unwrap();
    let outside = dir.join("elsewhere.png");
    std::fs::write(&outside, b"not the book's").unwrap();
    std::os::unix::fs::symlink(&outside, root.join("assets/linked.png")).unwrap();
    let (mut folder, read) = Folder::open(&root).unwrap();
    assert_eq!(read, files(), "the link is not read");
    folder.save(&read).unwrap();
    assert!(!pack::unpack(&pack::pack_folder(&root, &read).unwrap()).unwrap().files.assets.contains_key("linked.png"));
    assert_eq!(std::fs::read(&outside).unwrap(), b"not the book's");
}

#[test]
fn a_linked_book_toml_is_not_a_book() {
    let dir = tmp("manifest");
    let real = dir.join("Real.wsbk");
    Folder::create(&real, &files()).unwrap();
    let root = dir.join("A.wsbk");
    std::fs::create_dir_all(&root).unwrap();
    std::os::unix::fs::symlink(real.join("book.toml"), root.join("book.toml")).unwrap();
    assert_eq!(Folder::open(&root).unwrap_err(), Error::NotABook);
}
