// SPDX-License-Identifier: MIT OR Apache-2.0
//! The command line (plan M9.2, AUT-1): each command on the fixtures, with its exit code.
//! `command_line.rs` holds the program to the core's convention (which code, which output).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_wordsmith-book-cli")).args(args).output().unwrap()
}

fn text(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned() + &String::from_utf8_lossy(&o.stderr)
}

fn tmp(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn usage_and_unknown_commands_exit_with_two() {
    assert_eq!(cli(&[]).status.code(), Some(2));
    let o = cli(&["frobnicate"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(text(&o).contains("unknown command"));
    assert_eq!(cli(&["help"]).status.code(), Some(0));
    assert_eq!(cli(&["build"]).status.code(), Some(2));
    // A file that cannot be opened is a failure, not a wrong command line.
    assert_eq!(cli(&["build", "/no/such/book.wsbk"]).status.code(), Some(1));
    let o = cli(&["commands"]);
    assert_eq!(o.status.code(), Some(0));
    assert!(text(&o).contains("book.export") && text(&o).contains("book.preflight"));
}

#[test]
fn convert_report_run_check_build_and_verify_a_book() {
    let dir = tmp("cli");
    let project = dir.join("Gulliver.wsbk");
    let p = project.to_str().unwrap();
    // convert: a DOCX into a Book project.
    let o = cli(&["convert", root().join("tests/fixtures/novels/gullivers-travels.docx").to_str().unwrap(), p, "--title", "Gulliver's Travels"]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
    assert_eq!(cli(&["convert", "x.docx", p]).status.code(), Some(1), "an existing project is not overwritten");
    // report
    let o = cli(&["report", p]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
    let r: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(r["words"].as_u64().unwrap() > 100_000, "{r}");
    assert_eq!(r["title"], "Gulliver's Travels", "{r}");
    // run: any command; changes are saved.
    let o = cli(&["run", p, "book.design.choose", r#"{"id": "fantasy"}"#]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
    assert!(std::fs::read_to_string(project.join("book.toml")).unwrap().contains("fantasy"));
    assert_eq!(cli(&["run", p, "book.design.choose", r#"{"id": "none-such"}"#]).status.code(), Some(1));
    assert_eq!(cli(&["run", p, "book.design.choose", "{not json"]).status.code(), Some(2));
    // check: passes.
    let o = cli(&["check", p, "--edition", "ebook"]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
    assert!(text(&o).contains("nothing blocks export"));
    // build: the PDF and EPUB.
    let out = dir.join("out");
    let o = cli(&["build", p, "--out", out.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
    let files: Vec<PathBuf> = std::fs::read_dir(&out).unwrap().map(|e| e.unwrap().path()).collect();
    assert_eq!(files.len(), 2);
    // verify: each output, and the source document, against the book.
    for f in &files {
        let o = cli(&["verify", p, f.to_str().unwrap()]);
        assert_eq!(o.status.code(), Some(0), "{}: {}", f.display(), text(&o));
    }
    let docx = root().join("tests/fixtures/novels/gullivers-travels.docx");
    // The source document holds the book's words.
    let o = cli(&["verify", p, docx.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
    // A different book's words: the check ran and said no, so 3, with its answer as the result.
    let o = cli(&["verify", p, root().join("tests/fixtures/novels/waverley.txt").to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(3), "{}", text(&o));
    let said = String::from_utf8_lossy(&o.stdout);
    assert!(said.contains("differ") || said.contains("expected"), "{}", text(&o));
    assert!(o.stderr.is_empty(), "{}", text(&o));
    // A failing check: an edition with a wrong ISBN blocks; exit code 3 and nothing written.
    let toml = std::fs::read_to_string(project.join("book.toml")).unwrap();
    let broken = toml.replacen("id = \"paperback\"", "id = \"paperback\"\nisbn = \"978-0-306-40615-8\"", 1);
    assert_ne!(broken, toml);
    std::fs::write(project.join("book.toml"), broken).unwrap();
    let o = cli(&["check", p]);
    assert_eq!(o.status.code(), Some(3), "{}", text(&o));
    assert!(String::from_utf8_lossy(&o.stdout).contains("Blocks export") && o.stderr.is_empty(), "{}", text(&o));
    let out2 = dir.join("out2");
    let o = cli(&["build", p, "--out", out2.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(3), "{}", text(&o));
    let said = String::from_utf8_lossy(&o.stdout);
    assert!(said.contains("Blocks export") && said.contains("nothing was exported") && o.stderr.is_empty(), "{}", text(&o));
    assert!(!out2.exists());
}

#[test]
fn mcp_serves_the_tools_over_stdio() {
    use std::io::Write as _;
    let mut child = Command::new(env!("CARGO_BIN_EXE_wordsmith-book-cli"))
        .arg("mcp")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2025-06-18","capabilities":{{}},"clientInfo":{{"name":"t","version":"1"}}}}}}"#).unwrap();
    writeln!(stdin, r#"{{"jsonrpc":"2.0","id":2,"method":"tools/list"}}"#).unwrap();
    drop(stdin);
    let out = child.wait_with_output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("book_export") && s.contains("Wordsmith Book"), "{s}");
}
