// SPDX-License-Identifier: MIT OR Apache-2.0
//! `book-cli`, held to the core's convention for command lines and agent tools (core decision
//! 0017, the core's `docs/command-lines-and-agent-tools.md`): the program itself is run, and
//! its exit code, its standard output and its standard error are read.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use jcj_agents::cli::Exit;
use serde_json::{Value, json};

fn run(args: &[&str], input: &str) -> Output {
    let mut child =
        Command::new(env!("CARGO_BIN_EXE_book-cli")).args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    child.wait_with_output().unwrap()
}

fn code(o: &Output) -> Option<i32> {
    o.status.code()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn exit(e: Exit) -> Option<i32> {
    Some(i32::from(e.code()))
}

fn tmp(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn path(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

const TINY: &str = "Chapter One\n\nIt was a small book, and it knew it.\n\nChapter Two\n\nIt ended soon after.\n";

/// A short text and the Book project `convert` makes of it.
fn tiny_book(dir: &Path) -> (String, String) {
    let (file, project) = (dir.join("tiny.txt"), dir.join("Tiny.wsbk"));
    std::fs::write(&file, TINY).unwrap();
    let o = run(&["convert", &path(&file), &path(&project)], "");
    assert_eq!(code(&o), exit(Exit::Done), "{}", text(&o.stderr));
    (path(&file), path(&project))
}

#[test]
fn help_and_version_are_answered_on_standard_output() {
    for help in ["--help", "-h", "help"] {
        let o = run(&[help], "");
        assert_eq!(code(&o), exit(Exit::Done), "{help}");
        assert!(text(&o.stdout).starts_with("usage:") && o.stderr.is_empty(), "{help}");
        assert!(text(&o.stdout).contains("mcp [<file>] [--read-root <folder>] [--write-root <folder>]"));
    }
    for version in ["--version", "-V"] {
        let o = run(&[version], "");
        assert_eq!(code(&o), exit(Exit::Done), "{version}");
        assert_eq!(text(&o.stdout).lines().next(), Some(concat!("book-cli ", env!("CARGO_PKG_VERSION"))));
        assert!(o.stderr.is_empty(), "{version}");
    }
}

#[test]
fn a_wrong_command_line_is_a_usage_error_on_standard_error() {
    let wrong: [&[&str]; 13] = [
        // Nothing was asked, so nothing was done: not a success.
        &[],
        &["--bogus"],
        &["frobnicate"],
        &["build"],
        &["build", "a.wsbk", "--bogus"],
        &["build", "a.wsbk", "--out"],
        &["verify", "one"],
        &["commands", "extra"],
        &["run", "a.wsbk", "book.design.choose", "{not json"],
        &["mcp", "--bogus"],
        &["mcp", "--read-root"],
        &["mcp", "a.wsbk", "b.wsbk"],
        // A root that is no folder is a wrong command line too, said before a project is opened.
        &["mcp", "--read-root", "no-such-folder-for-book-cli"],
    ];
    for bad in wrong {
        let o = run(bad, "");
        assert_eq!(code(&o), exit(Exit::Usage), "{bad:?}");
        assert!(o.stdout.is_empty(), "{bad:?}: nothing on standard output");
        assert!(text(&o.stderr).starts_with("book-cli: ") && text(&o.stderr).contains("usage:"), "{bad:?}: {}", text(&o.stderr));
    }
}

#[test]
fn what_could_not_be_done_is_a_failure_on_standard_error() {
    let dir = tmp("command-line-failures");
    let (file, project) = tiny_book(&dir);
    let missing = path(&dir.join("No Such.wsbk"));
    let unreadable = path(&dir.join("picture.xyz"));
    std::fs::write(&unreadable, "not a book").unwrap();
    let failures: [&[&str]; 9] = [
        // A project that cannot be opened.
        &["build", &missing],
        &["check", &missing],
        &["report", &missing],
        &["verify", &missing, &file],
        &["run", &missing, "book.outline"],
        &["mcp", &missing],
        // A file that cannot be read, and one Book cannot check.
        &["verify", &project, &path(&dir.join("missing.txt"))],
        &["verify", &project, &unreadable],
        // A project that is there already is not written over.
        &["convert", &file, &project],
    ];
    for failed in failures {
        let o = run(failed, "");
        assert_eq!(code(&o), exit(Exit::Failed), "{failed:?}: {}", text(&o.stderr));
        assert!(o.stdout.is_empty(), "{failed:?}: nothing on standard output");
        let said = text(&o.stderr);
        assert!(said.starts_with("book-cli: ") && !said.contains("usage:"), "{failed:?}: {said}");
    }
    // A command that refuses, and an import that cannot be read.
    let o = run(&["run", &project, "book.design.choose", r#"{"id": "none-such"}"#], "");
    assert_eq!(code(&o), exit(Exit::Failed));
    assert!(o.stdout.is_empty() && text(&o.stderr).starts_with("book-cli: "), "{}", text(&o.stderr));
    let o = run(&["convert", &unreadable, &path(&dir.join("Other.wsbk"))], "");
    assert_eq!(code(&o), exit(Exit::Failed));
    assert!(text(&o.stderr).starts_with("book-cli: "), "{}", text(&o.stderr));
}

#[test]
fn a_check_that_does_not_pass_is_an_answer_not_a_failure() {
    let dir = tmp("command-line-checks");
    let (file, project) = tiny_book(&dir);
    // verify: the same words, then other words.
    let o = run(&["verify", &project, &file], "");
    assert_eq!(code(&o), exit(Exit::Done), "{}", text(&o.stderr));
    assert!(text(&o.stdout).contains("every word matches"), "{}", text(&o.stdout));
    let other = dir.join("other.txt");
    std::fs::write(&other, TINY.replace("soon after", "much later")).unwrap();
    let o = run(&["verify", &project, &path(&other)], "");
    assert_eq!(code(&o), exit(Exit::Differs));
    assert!(text(&o.stdout).contains("other.txt") && o.stderr.is_empty(), "{} / {}", text(&o.stdout), text(&o.stderr));
    // check and build: an edition with an ISBN that is no ISBN blocks export.
    let toml = std::fs::read_to_string(dir.join("Tiny.wsbk/book.toml")).unwrap();
    let broken = toml.replacen("id = \"paperback\"", "id = \"paperback\"\nisbn = \"978-0-306-40615-8\"", 1);
    assert_ne!(broken, toml);
    std::fs::write(dir.join("Tiny.wsbk/book.toml"), broken).unwrap();
    let o = run(&["check", &project, "--edition", "paperback"], "");
    assert_eq!(code(&o), exit(Exit::Differs), "{}", text(&o.stderr));
    assert!(text(&o.stdout).contains("export is blocked") && text(&o.stdout).contains("Blocks export") && o.stderr.is_empty(), "{}", text(&o.stdout));
    let out = dir.join("out");
    let o = run(&["build", &project, "--edition", "paperback", "--out", &path(&out)], "");
    assert_eq!(code(&o), exit(Exit::Differs), "{}", text(&o.stderr));
    assert!(text(&o.stdout).contains("nothing was exported") && o.stderr.is_empty(), "{}", text(&o.stdout));
    assert!(!out.exists(), "a blocked export writes nothing");
    // A build that cannot run is a failure: there is no such edition.
    let o = run(&["build", &project, "--edition", "no-such-edition", "--out", &path(&out)], "");
    assert_eq!(code(&o), exit(Exit::Failed));
    assert!(o.stdout.is_empty() && text(&o.stderr).starts_with("book-cli: "), "{}", text(&o.stderr));
}

fn request(id: u64, method: &str, params: Value) -> String {
    json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}).to_string() + "\n"
}

fn tool(id: u64, name: &str, arguments: Value) -> String {
    request(id, "tools/call", json!({"name": name, "arguments": arguments}))
}

fn replies(o: &Output) -> Vec<Value> {
    text(&o.stdout).lines().map(|l| serde_json::from_str(l).unwrap()).collect()
}

fn errors(o: &Output) -> Vec<Option<bool>> {
    replies(o).iter().map(|r| r["result"]["isError"].as_bool()).collect()
}

#[test]
fn mcp_serves_books_own_tools_until_its_input_ends() {
    // Unconfined: said on standard error; standard output carries only the protocol.
    let o = run(&["mcp"], &(request(1, "initialize", json!({})) + &request(2, "tools/list", json!({}))));
    assert_eq!(code(&o), exit(Exit::Done));
    assert!(text(&o.stderr).contains("no --read-root or --write-root given"), "{}", text(&o.stderr));
    let replies = replies(&o);
    assert_eq!(replies.len(), 2, "one line of JSON for each request, nothing else");
    assert_eq!(replies[0]["result"]["serverInfo"]["name"], "wordsmith-book");
    let names: Vec<&str> = replies[1]["result"]["tools"].as_array().unwrap().iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(names.contains(&"book_export") && names.contains(&"book_open") && names.contains(&"book_save"), "{names:?}");
    assert!(names.iter().all(|n| n.starts_with("book_")), "every tool is Book's own: {names:?}");
}

#[test]
fn mcp_keeps_an_agent_to_the_folders_given() {
    let dir = tmp("command-line-mcp");
    let (work, out, elsewhere) = (dir.join("work"), dir.join("work/out"), dir.join("elsewhere"));
    std::fs::create_dir_all(&out).unwrap();
    std::fs::create_dir_all(&elsewhere).unwrap();
    std::fs::write(work.join("tiny.txt"), TINY).unwrap();
    // The agent's own book: made inside the write folder, and nowhere else.
    let import = |id: u64, to: &Path| tool(id, "book_import", json!({"file": path(&work.join("tiny.txt")), "path": path(to)}));
    let pack = |id: u64, to: &Path| tool(id, "book_saveAsOneFile", json!({"path": path(to)}));
    let input = import(1, &elsewhere.join("A.wsbk"))
        + &import(2, &out.join("A.wsbk"))
        + &pack(3, &out.join("A.wsbkz"))
        + &pack(4, &elsewhere.join("A.wsbkz"));
    let o = run(&["mcp", "--read-root", &path(&work), "--write-root", &path(&out)], &input);
    assert_eq!(code(&o), exit(Exit::Done));
    assert!(!text(&o.stderr).contains("any file"), "{}", text(&o.stderr));
    assert_eq!(errors(&o), [Some(true), Some(false), Some(false), Some(true)], "{}", text(&o.stdout));
    assert!(out.join("A.wsbk/book.toml").is_file() && out.join("A.wsbkz").is_file());
    assert!(!elsewhere.join("A.wsbk").exists() && !elsewhere.join("A.wsbkz").exists());

    // The book named on the command line is the person's: it lies outside both folders, and is
    // changed and saved where it lives. What the agent names is still held to the folders.
    let (_, own) = tiny_book(&elsewhere);
    let input = tool(1, "book_design_choose", json!({"id": "romance"}))
        + &tool(2, "book_save", json!({}))
        + &pack(3, &elsewhere.join("Own.wsbkz"))
        + &pack(4, &out.join("Own.wsbkz"));
    let o = run(&["mcp", &own, "--read-root", &path(&work), "--write-root", &path(&out)], &input);
    assert_eq!(code(&o), exit(Exit::Done));
    assert_eq!(errors(&o), [Some(false), Some(false), Some(true), Some(false)], "{}", text(&o.stdout));
    assert!(std::fs::read_to_string(elsewhere.join("Tiny.wsbk/book.toml")).unwrap().contains("romance"));
    assert!(out.join("Own.wsbkz").is_file() && !elsewhere.join("Own.wsbkz").exists());

    // With --read-root alone, nothing can be written.
    let o = run(&["mcp", &own, "--read-root", &path(&work)], &pack(1, &out.join("No.wsbkz")));
    assert_eq!(errors(&o), [Some(true)], "{}", text(&o.stdout));
    assert!(!out.join("No.wsbkz").exists());
}
