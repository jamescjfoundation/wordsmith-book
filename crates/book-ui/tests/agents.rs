// SPDX-License-Identifier: MIT OR Apache-2.0
//! Agent tools (plan M9.1, AUT-2; core decision 0017): every Book command is a tool named
//! `book_<command>`, and nothing else is; a book is imported, designed, built and checked
//! through the tools alone; and an agent given folders is kept to them.

use std::path::{Path, PathBuf};

use jcj_agents::{AgentServer, app_tool_name};
use jcj_engine::file_access::AutomationRoots;
use serde_json::{Value, json};
use wordsmith_book_ui::agents::{ALIASES, PREFIX, WINDOW_ONLY, server, tool_names};
use wordsmith_book_ui::{BookApp, commands};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn tmp(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn text(p: &Path) -> String {
    p.display().to_string()
}

/// Call a tool; gives its result's text parsed as JSON (or as a string), or the error text.
fn call(s: &mut AgentServer, id: &mut u64, name: &str, args: Value) -> Result<Value, String> {
    *id += 1;
    let out = s.handle(json!({ "jsonrpc": "2.0", "id": *id, "method": "tools/call", "params": { "name": name, "arguments": args } })).unwrap();
    let result = out.get("result").cloned().ok_or_else(|| out.to_string())?;
    let text = result["content"][0]["text"].as_str().unwrap_or("").to_owned();
    if result["isError"] == true {
        return Err(text);
    }
    Ok(serde_json::from_str(&text).unwrap_or(Value::String(text)))
}

#[test]
fn every_book_command_is_a_tool_and_nothing_else_is() {
    let mut s = server(BookApp::new(), "0.1.0", None);
    let init = s.handle(json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "test", "version": "1" } } })).unwrap();
    assert_eq!(init["result"]["serverInfo"]["name"], "wordsmith-book");
    assert!(init["result"]["instructions"].as_str().unwrap().contains("book_export"));
    let list = s.handle(json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" })).unwrap();
    let tools = list["result"]["tools"].as_array().unwrap();
    let listed: Vec<String> = tools.iter().map(|t| t["name"].as_str().unwrap().to_owned()).collect();
    let mut want: Vec<String> = Vec::new();
    for c in commands::list() {
        if c.id.starts_with("book.") {
            want.push(app_tool_name(PREFIX, &c.id));
        } else if let Some((alias, _, _)) = ALIASES.iter().find(|(_, shell, _)| *shell == c.id) {
            want.push(app_tool_name(PREFIX, alias));
        } else {
            assert!(WINDOW_ONLY.iter().any(|(w, _)| *w == c.id), "{} has no tool", c.id);
        }
    }
    assert_eq!(listed, s.tool_names(), "the tools listed are the app's own, and no general document tool");
    assert_eq!(listed, tool_names());
    assert!(listed.iter().all(|t| t.starts_with("book_")), "{listed:?}");
    let mut got = listed.clone();
    got.sort();
    want.sort();
    assert_eq!(got, want, "the Book tools are exactly the Book commands");
    // A tool says whether it only reads, and gives its command and parameters.
    let tool = |name: &str| tools.iter().find(|t| t["name"] == name).unwrap();
    assert_eq!(tool("book_outline")["annotations"]["readOnlyHint"], true);
    assert_eq!(tool("book_export")["annotations"]["readOnlyHint"], false);
    assert!(tool("book_export")["description"].as_str().unwrap().contains("Command `book.export`. Parameters: {\"folder\""));
    // The general document tools are not served: there is no document behind them.
    let mut id = 2;
    let e = call(&mut s, &mut id, "execute", json!({ "command": "book.outline" })).unwrap_err();
    assert!(e.contains("unknown tool"), "{e}");
}

#[test]
fn a_book_is_imported_designed_built_and_checked_through_tools_alone() {
    let dir = tmp("agents");
    let mut s = server(BookApp::new(), "0.1.0", None);
    let mut id = 10;
    let file = root().join("tests/fixtures/novels/waverley.docx");
    let report =
        call(&mut s, &mut id, "book_import", json!({ "file": file.display().to_string(), "path": dir.join("Waverley.wsbk").display().to_string() }))
            .unwrap();
    assert!(report.get("check").is_some() || report.is_object(), "{report}");
    let outline = call(&mut s, &mut id, "book_outline", json!({})).unwrap();
    assert!(outline.as_array().is_some_and(|v| v.len() > 20), "{outline}");
    call(&mut s, &mut id, "book_design_choose", json!({ "id": "romance" })).unwrap();
    call(&mut s, &mut id, "book_metadata_set", json!({ "field": "byline", "value": "Walter Scott" })).unwrap();
    let editions = call(&mut s, &mut id, "book_metadata", json!({})).unwrap();
    let paperback = editions["editions"].as_array().unwrap().iter().find(|e| e["format"] == "paperback").unwrap()["id"].as_str().unwrap().to_owned();
    call(&mut s, &mut id, "book_edition_set", json!({ "id": paperback, "trim": "6x9" })).unwrap();
    let check = call(&mut s, &mut id, "book_preflight", json!({ "edition": paperback })).unwrap();
    assert_eq!(check["blocked"], false, "{check}");
    let out = dir.join("out");
    let files = call(&mut s, &mut id, "book_export", json!({ "folder": out.display().to_string() })).unwrap();
    let files = files["files"].as_array().unwrap();
    assert_eq!(files.len(), 2, "{files:?}");
    assert!(files.iter().all(|f| Path::new(f.as_str().unwrap()).is_file()));
    call(&mut s, &mut id, "book_save", json!({})).unwrap();
    // A command that fails says why.
    let err = call(&mut s, &mut id, "book_design_choose", json!({ "id": "no-such-design" })).unwrap_err();
    assert!(!err.is_empty());
}

/// A folder the agent may read (`work`), one inside it that it may also write (`work/out`), and
/// one it may not touch (`elsewhere`), with a short text to import in each of the outer two.
struct Folders {
    work: PathBuf,
    out: PathBuf,
    elsewhere: PathBuf,
}

fn folders(name: &str) -> (Folders, Option<std::sync::Arc<AutomationRoots>>) {
    let dir = tmp(name);
    let f = Folders { work: dir.join("work"), out: dir.join("work/out"), elsewhere: dir.join("elsewhere") };
    for d in [&f.out, &f.elsewhere] {
        std::fs::create_dir_all(d).unwrap();
    }
    for d in [&f.work, &f.elsewhere] {
        std::fs::write(d.join("tiny.txt"), "Chapter One\n\nIt was a small book, and it knew it.\n\nChapter Two\n\nIt ended soon after.\n").unwrap();
    }
    let roots = AutomationRoots::new(Some(&f.work), Some(&f.out)).unwrap();
    assert!(roots.is_some());
    (f, roots)
}

#[test]
fn an_agent_given_folders_reads_and_writes_only_inside_them() {
    let (f, roots) = folders("agents-folders");
    let mut s = server(BookApp::new(), "0.1.0", roots);
    let mut id = 0;
    let refused = |r: Result<Value, String>, what: &str| {
        let e = r.expect_err(what);
        assert!(e.contains("outside"), "{what}: {e}");
    };
    // Importing: the file inside the read folder, the new book inside the write folder.
    let import = |file: &Path, path: Option<&Path>| match path {
        Some(p) => json!({ "file": text(file), "path": text(p) }),
        None => json!({ "file": text(file) }),
    };
    refused(
        call(&mut s, &mut id, "book_import", import(&f.elsewhere.join("tiny.txt"), Some(&f.out.join("A.wsbk")))),
        "a file outside the read folder",
    );
    refused(
        call(&mut s, &mut id, "book_import", import(&f.work.join("tiny.txt"), Some(&f.elsewhere.join("A.wsbk")))),
        "a book outside the write folder",
    );
    refused(call(&mut s, &mut id, "book_import", import(&f.work.join("tiny.txt"), None)), "a book beside a file that is outside the write folder");
    assert!(!f.elsewhere.join("A.wsbk").exists() && !f.work.join("tiny.wsbk").exists() && !f.out.join("A.wsbk").exists());
    call(&mut s, &mut id, "book_import", import(&f.work.join("tiny.txt"), Some(&f.out.join("A.wsbk")))).unwrap();
    // The agent's own book lies in the write folder: it is changed and saved there.
    call(&mut s, &mut id, "book_design_choose", json!({ "id": "romance" })).unwrap();
    call(&mut s, &mut id, "book_save", json!({})).unwrap();
    assert!(std::fs::read_to_string(f.out.join("A.wsbk/book.toml")).unwrap().contains("romance"));
    // Files it writes: where a parameter says, and where Book writes when none does.
    call(&mut s, &mut id, "book_design_export", json!({ "path": text(&f.out.join("design.toml")) })).unwrap();
    refused(
        call(&mut s, &mut id, "book_design_export", json!({ "path": text(&f.elsewhere.join("design.toml")) })),
        "a file outside the write folder",
    );
    refused(call(&mut s, &mut id, "book_design_export", json!({ "path": text(&f.work.join("design.toml")) })), "the read folder is not written");
    assert!(f.out.join("design.toml").is_file() && !f.elsewhere.join("design.toml").exists() && !f.work.join("design.toml").exists());
    let onix = call(&mut s, &mut id, "book_onix", json!({})).unwrap();
    assert!(Path::new(onix["file"].as_str().unwrap()).starts_with(&f.out), "the export folder beside the book: {onix}");
    // A linked manuscript is named from the new book's folder, and read there.
    let linked = json!({ "path": text(&f.out.join("B.wsbk")), "manuscript": "../../../elsewhere/My Novel.wsm" });
    refused(call(&mut s, &mut id, "book_new", linked), "a manuscript outside the read folder");
    assert!(!f.out.join("B.wsbk").exists());
    // One inside the read folder is allowed, by the same path Book will read it from.
    let inside = json!({ "path": text(&f.out.join("B.wsbk")), "manuscript": "../../My Novel.wsm" });
    call(&mut s, &mut id, "book_new", inside).unwrap();
    assert!(std::fs::read_to_string(f.out.join("B.wsbk/book.toml")).unwrap().contains("../../My Novel.wsm"));
    // Starting another program cannot be kept to a folder; the ebook alone goes to a folder
    // of Book's own, which is not the agent's.
    let e = call(&mut s, &mut id, "book_ebook_openInReadingApp", json!({})).unwrap_err();
    assert!(e.contains("reading app"), "{e}");
    refused(call(&mut s, &mut id, "book_ebook_openInReadingApp", json!({ "open": false })), "the temporary folder");

    // A book in the read folder only: read, and its files written elsewhere, but not changed.
    let copy = f.work.join("Read Only.wsbk");
    copy_folder(&f.out.join("A.wsbk"), &copy);
    let before = std::fs::read(copy.join("book.toml")).unwrap();
    call(&mut s, &mut id, "book_open", json!({ "path": text(&copy) })).unwrap();
    assert!(call(&mut s, &mut id, "book_outline", json!({})).unwrap().as_array().is_some_and(|rows| !rows.is_empty()));
    call(&mut s, &mut id, "book_design_export", json!({ "path": text(&f.out.join("read-only.toml")) })).unwrap();
    for (tool, args) in [
        ("book_design_choose", json!({ "id": "fantasy" })),
        ("book_metadata_set", json!({ "field": "byline", "value": "Someone" })),
        ("book_save", json!({})),
        ("book_backup", json!({})),
        ("book_undo", json!({})),
    ] {
        refused(call(&mut s, &mut id, tool, args), tool);
    }
    assert_eq!(std::fs::read(copy.join("book.toml")).unwrap(), before);
    assert!(!f.work.join("Read Only Backups").exists());
    // A book in the read folder that links a manuscript outside it is not kept open.
    copy_folder(&root().join("tests/fixtures/novels/gullivers-travels.wsm"), &f.elsewhere.join("My Novel.wsm"));
    let linked = f.work.join("Linked.wsbk");
    BookApp::new().command(commands::NEW, &json!({ "path": text(&linked), "manuscript": text(&f.elsewhere.join("My Novel.wsm")) })).unwrap();
    let e = call(&mut s, &mut id, "book_open", json!({ "path": text(&linked) })).unwrap_err();
    assert!(e.contains("links a manuscript") && e.contains("outside"), "{e}");
    assert!(call(&mut s, &mut id, "book_outline", json!({})).unwrap_err().contains("No book is open"));
    // A book outside the read folder is not opened.
    copy_folder(&f.out.join("A.wsbk"), &f.elsewhere.join("Other.wsbk"));
    refused(call(&mut s, &mut id, "book_open", json!({ "path": text(&f.elsewhere.join("Other.wsbk")) })), "a book outside the read folder");
}

#[test]
fn the_book_the_server_was_started_on_is_saved_where_it_lives() {
    let (f, roots) = folders("agents-own-book");
    // The person's book lies outside both folders: they named it, so it is theirs to name.
    let own = f.elsewhere.join("Own.wsbk");
    let mut app = BookApp::new();
    app.command(commands::IMPORT, &json!({ "file": text(&f.elsewhere.join("tiny.txt")), "path": text(&own) })).unwrap();
    let mut s = server(app, "0.1.0", roots);
    let mut id = 0;
    call(&mut s, &mut id, "book_design_choose", json!({ "id": "romance" })).unwrap();
    call(&mut s, &mut id, "book_save", json!({})).unwrap();
    assert!(std::fs::read_to_string(own.join("book.toml")).unwrap().contains("romance"));
    // What it writes besides the project still goes where the agent may write.
    let e = call(&mut s, &mut id, "book_onix", json!({})).unwrap_err();
    assert!(e.contains("outside"), "the export folder beside the book is not the agent's: {e}");
    assert!(!f.elsewhere.join("Own Export").exists());
    let onix = call(&mut s, &mut id, "book_onix", json!({ "folder": text(&f.out.join("own")) })).unwrap();
    assert!(Path::new(onix["file"].as_str().unwrap()).is_file(), "{onix}");
    // Another book is the agent's choice, and is held to the folders.
    copy_folder(&own, &f.elsewhere.join("Other.wsbk"));
    let e = call(&mut s, &mut id, "book_open", json!({ "path": text(&f.elsewhere.join("Other.wsbk")) })).unwrap_err();
    assert!(e.contains("outside"), "{e}");
    // The refused open left the person's book open.
    call(&mut s, &mut id, "book_save", json!({})).unwrap();
}

fn copy_folder(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let (p, dest) = (e.path(), to.join(e.file_name()));
        if p.is_dir() {
            copy_folder(&p, &dest);
        } else {
            std::fs::copy(&p, &dest).unwrap();
        }
    }
}
