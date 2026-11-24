// SPDX-License-Identifier: MIT OR Apache-2.0
//! Agent tools (plan M9.1, AUT-2): every Book command is a tool, and a book is imported,
//! designed, built and checked through the tools alone.

use std::path::{Path, PathBuf};

use jcj_agents::tool_name;
use serde_json::{Value, json};
use wordsmith_book_ui::agents::{ALIASES, WINDOW_ONLY, server};
use wordsmith_book_ui::{BookApp, commands};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Call a tool; gives its result's text parsed as JSON (or as a string), or the error text.
fn call(s: &mut jcj_agents::AgentServer, id: &mut u64, name: &str, args: Value) -> Result<Value, String> {
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
fn every_book_command_is_a_tool() {
    let mut s = server(BookApp::new(), "0.1.0");
    let init = s.handle(json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "test", "version": "1" } } })).unwrap();
    assert!(init["result"]["instructions"].as_str().unwrap().contains("book_export"));
    let list = s.handle(json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" })).unwrap();
    let tools: Vec<String> = list["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap().to_owned()).collect();
    let app_tools: Vec<String> = s.app_tools().iter().map(|t| t.name.clone()).collect();
    let mut want: Vec<String> = Vec::new();
    for c in commands::list() {
        if c.id.starts_with("book.") {
            want.push(tool_name(&c.id));
        } else if let Some((alias, _, _)) = ALIASES.iter().find(|(_, shell, _)| *shell == c.id) {
            want.push(tool_name(alias));
        } else {
            assert!(WINDOW_ONLY.iter().any(|(w, _)| *w == c.id), "{} has no tool", c.id);
        }
    }
    let mut got = app_tools.clone();
    got.sort();
    want.sort();
    assert_eq!(got, want, "the Book tools are exactly the Book commands");
    for t in &app_tools {
        assert!(tools.contains(t), "{t} is listed");
    }
}

#[test]
fn a_book_is_imported_designed_built_and_checked_through_tools_alone() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("agents");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut s = server(BookApp::new(), "0.1.0");
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
