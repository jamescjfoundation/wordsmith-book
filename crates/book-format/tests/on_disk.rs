// SPDX-License-Identifier: MIT OR Apache-2.0
//! The files on disk are the judge: what a save, a packed project, a recovery copy, a backup and
//! an unpacked project write is compared, byte for byte (by its SHA-256), with
//! `tests/golden/project-files.txt`.
//!
//! The reference was written by Wordsmith Book 0.9.0's own folder code, before the folder layer
//! became the core's (`jcj_project::folder`, core issue #45), so this test proves that the move
//! changed no byte a user keeps. A change that alters the reference is deliberate: rerun with
//! `UPDATE_GOLDEN=1`, review the difference, and record it in `docs/golden.md`.

#[path = "on_disk/scenario.rs"]
mod scenario;

use std::path::Path;

#[test]
fn the_files_on_disk_are_the_reference_bytes() {
    let got = scenario::written(&Path::new(env!("CARGO_TARGET_TMPDIR")).join("on-disk"));
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/project-files.txt");
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(&path, &got).unwrap();
    }
    let want = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{} is missing: run with UPDATE_GOLDEN=1 and review it", path.display()));
    assert_eq!(got, want, "a file a project writes changed: review it and record it in docs/golden.md");
}
