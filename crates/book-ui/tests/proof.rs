// SPDX-License-Identifier: MIT OR Apache-2.0
//! Proofs from the app (plan R3.6, PRT-13): `book.proof` writes a print edition's proof beside
//! the exports, named as a proof; an ebook has none; the export page offers proofs apart from the
//! export itself.

use std::path::Path;

use serde_json::json;
use wordsmith_book_ui::{BookApp, commands};

#[test]
fn a_proof_is_written_when_asked_for_and_only_then() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("proof-command");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("tale.txt");
    std::fs::write(&file, "CHAPTER I.\n\nThe lamp burned low over the harbour, and the ships slept.\n").unwrap();
    let mut app = BookApp::new();
    app.command(commands::IMPORT, &json!({ "file": file.display().to_string(), "path": dir.join("T.wsbk").display().to_string() })).unwrap();
    let out = dir.join("out");
    let v = app.command(commands::PROOF, &json!({ "edition": "paperback", "folder": out.display().to_string() })).unwrap();
    let written = Path::new(v["file"].as_str().unwrap());
    assert!(written.is_file() && written.starts_with(&out));
    assert!(written.file_name().unwrap().to_string_lossy().ends_with("Paperback proof.pdf"), "{}", written.display());
    assert!(std::fs::read(written).unwrap().starts_with(b"%PDF-"));
    assert!(app.command(commands::PROOF, &json!({ "edition": "ebook", "folder": out.display().to_string() })).is_err(), "an ebook has no proof");
    // The export page's default: no proof is asked for.
    assert!(app.export.proof_request.is_none());
    assert_eq!(std::fs::read_dir(&out).unwrap().count(), 1, "nothing but the proof asked for");
}
