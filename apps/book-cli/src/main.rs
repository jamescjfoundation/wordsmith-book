// SPDX-License-Identifier: MIT OR Apache-2.0
//! Wordsmith Book on the command line (AUT-1): build, check, verify, convert, report, run,
//! commands and mcp. Every command goes through the same code as the window and the agent
//! tools.
//!
//! Exit codes, for scripts: 0 done; 1 a check failed (export blocked, words differ, a command
//! refused); 2 the command line was wrong or a file could not be opened.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::path::Path;
use std::process::ExitCode;

use serde_json::{Value, json};
use wordsmith_book_check::preflight::{Finding, Link};
use wordsmith_book_ui::{BookApp, commands};

const USAGE: &str = "usage: book-cli <command> [arguments]

  build <project> [--edition <id>]... [--out <folder>] [--bundle]
        Build editions (all when none is named), check them, and write the files.
  check <project> [--edition <id>]...
        Preflight editions (all when none is named); prints what it finds.
  verify <project> <file>
        Check a PDF, EPUB or document against the project's words.
  convert <file> <project folder> [--title <title>]
        Import a DOCX, ODT, RTF, Markdown or text file into a new Book project.
  report <project>
        The book's title, author, editions, sections, words and needs, as JSON.
  run <project> <command id> [<JSON parameters>]
        Run any Book command (see `commands`) and print what it gives, as JSON.
  commands
        Every Book command, with its label.
  mcp [<project>]
        Serve Book's commands as agent tools (Model Context Protocol over stdio).

Exit codes: 0 done, 1 a check failed or a command refused, 2 a usage or file error.";

/// A failure and its exit code.
struct Fail(u8, String);

fn usage(why: &str) -> Fail {
    Fail(2, format!("{why}\n\n{USAGE}"))
}

fn open(path: &str) -> Result<BookApp, Fail> {
    let mut app = BookApp::new();
    app.open_path(path).map_err(|e| Fail(2, format!("{path}: {e}")))?;
    if let Some(Err(e)) = app.book.as_ref().map(|b| &b.source) {
        return Err(Fail(2, format!("{path}: the source could not be read: {e}")));
    }
    Ok(app)
}

/// `--name value` options (repeatable) and the rest, in order.
fn options(args: &[String], names: &[&str], flags: &[&str]) -> Result<(Vec<(String, String)>, Vec<String>, Vec<String>), Fail> {
    let (mut opts, mut set, mut rest) = (Vec::new(), Vec::new(), Vec::new());
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if let Some(name) = a.strip_prefix("--") {
            if flags.contains(&name) {
                set.push(name.to_owned());
            } else if names.contains(&name) {
                let v = args.get(i + 1).ok_or_else(|| usage(&format!("--{name} needs a value")))?;
                opts.push((name.to_owned(), v.clone()));
                i += 1;
            } else {
                return Err(usage(&format!("unknown option --{name}")));
            }
        } else {
            rest.push(a.clone());
        }
        i += 1;
    }
    Ok((opts, set, rest))
}

fn editions(app: &BookApp, opts: &[(String, String)]) -> Vec<String> {
    let named: Vec<String> = opts.iter().filter(|(k, _)| k == "edition").map(|(_, v)| v.clone()).collect();
    if named.is_empty() {
        app.book.as_ref().map(|b| b.files.project.editions.iter().map(|e| e.id.clone()).collect()).unwrap_or_default()
    } else {
        named
    }
}

fn line(f: &Finding) -> String {
    let link = match &f.link {
        Link::None => String::new(),
        Link::Page(n) => format!(" [page {n}]"),
        Link::Section(s) => format!(" [section {s}]"),
        Link::Edition(e) => format!(" [edition {e}]"),
        Link::File(p) => format!(" [{p}]"),
    };
    format!("{}: {}{link}", f.severity.title(), f.message)
}

fn build(args: &[String]) -> Result<(), Fail> {
    let (opts, flags, rest) = options(args, &["edition", "out"], &["bundle"])?;
    let [project] = rest.as_slice() else { return Err(usage("build takes one project")) };
    let mut app = open(project)?;
    let eds = editions(&app, &opts);
    let mut params = json!({ "editions": eds, "bundle": flags.iter().any(|f| f == "bundle") });
    if let Some((_, out)) = opts.iter().find(|(k, _)| k == "out") {
        params["folder"] = json!(out);
    }
    let v = app.command(commands::EXPORT, &params).map_err(|e| Fail(1, e))?;
    for f in v["files"].as_array().into_iter().flatten() {
        println!("{}", f.as_str().unwrap_or(""));
    }
    Ok(())
}

fn check(args: &[String]) -> Result<(), Fail> {
    let (opts, _, rest) = options(args, &["edition"], &[])?;
    let [project] = rest.as_slice() else { return Err(usage("check takes one project")) };
    let app = open(project)?;
    let b = app.book.as_ref().ok_or_else(|| Fail(2, "no book".into()))?;
    let validator = app.prefs.validator_settings();
    let mut blocked = false;
    for e in editions(&app, &opts) {
        let r = wordsmith_book_ui::preflight::run(b, &e, &validator).map_err(|m| Fail(2, m))?;
        println!("{} — {}", r.edition, if r.blocked() { "export is blocked" } else { "nothing blocks export" });
        for f in &r.findings {
            println!("  {}", line(f));
        }
        blocked |= r.blocked();
    }
    if blocked { Err(Fail(1, "preflight found problems that block export".into())) } else { Ok(()) }
}

fn verify(args: &[String]) -> Result<(), Fail> {
    let (_, _, rest) = options(args, &[], &[])?;
    let [project, file] = rest.as_slice() else { return Err(usage("verify takes a project and a file")) };
    let app = open(project)?;
    let b = app.book.as_ref().ok_or_else(|| Fail(2, "no book".into()))?;
    let src = b.source.as_ref().map_err(|e| Fail(2, e.clone()))?;
    let book = wordsmith_book_format::fixes::book_for_build(&b.files.project.fixes, src);
    let bytes = std::fs::read(file).map_err(|e| Fail(2, format!("{file}: {e}")))?;
    let ext = Path::new(file).extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    let source_words = wordsmith_book_check::verify::book_words(&book, true);
    let differs = |found: &[String]| match wordsmith_book_check::verify::compare(&source_words, found) {
        Ok(n) => {
            println!("every word matches ({n} words)");
            Ok(())
        }
        Err(d) => Err(Fail(1, format!("{file}: {d}"))),
    };
    match ext.as_str() {
        "epub" => differs(&wordsmith_book_check::epub::words(&bytes, true).map_err(|e| Fail(2, e))?),
        "pdf" => {
            // A PDF is checked against this project's own layout of its print edition.
            let edition =
                b.files.project.editions.iter().find(|e| e.format.is_print()).ok_or_else(|| Fail(2, "the book has no print edition".into()))?;
            let print =
                wordsmith_book_check::build::print(&b.files.project, &b.files.designs, &book, edition, &|p| b.picture(p)).map_err(|e| Fail(2, e))?;
            let found = wordsmith_book_check::preflight::print_words(&print, &bytes, &book);
            match found.first() {
                None => {
                    println!("every letter matches");
                    Ok(())
                }
                Some(f) => Err(Fail(1, format!("{file}: {}", f.message))),
            }
        }
        _ => {
            let format = match ext.as_str() {
                "docx" => wordsmith_book_import::Format::Docx,
                "odt" => wordsmith_book_import::Format::Odt,
                "rtf" => wordsmith_book_import::Format::Rtf,
                "md" | "markdown" => wordsmith_book_import::Format::Markdown,
                "txt" => wordsmith_book_import::Format::Text,
                _ => return Err(usage(&format!("{file}: Book reads PDF, EPUB, DOCX, ODT, RTF, Markdown and text files"))),
            };
            // A document is read as Book imports it, so its words compare with the book's.
            let i = wordsmith_book_import::import(format, &bytes, &b.files.project.cleanup.rules, "").map_err(|e| Fail(2, e.to_string()))?;
            differs(&wordsmith_book_check::verify::book_words(&wordsmith_manuscript_format::book::from_project(&i.project), true))
        }
    }
}

fn convert(args: &[String]) -> Result<(), Fail> {
    let (opts, _, rest) = options(args, &["title"], &[])?;
    let [file, project] = rest.as_slice() else { return Err(usage("convert takes a file and a project folder")) };
    if Path::new(project).exists() {
        return Err(Fail(2, format!("{project} already exists")));
    }
    let mut app = BookApp::new();
    let v = app.command(commands::IMPORT, &json!({ "file": file, "path": project })).map_err(|e| Fail(1, e))?;
    if let Some((_, t)) = opts.iter().find(|(k, _)| k == "title") {
        app.command(commands::METADATA_SET, &json!({ "field": "title", "value": t })).map_err(|e| Fail(1, e))?;
        app.book.as_mut().map(|b| b.save()).transpose().map_err(|e| Fail(1, e))?;
    }
    println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
    Ok(())
}

fn report(args: &[String]) -> Result<(), Fail> {
    let (_, _, rest) = options(args, &[], &[])?;
    let [project] = rest.as_slice() else { return Err(usage("report takes one project")) };
    let mut app = open(project)?;
    let meta = app.command(commands::METADATA, &json!({})).map_err(|e| Fail(1, e))?;
    let outline = app.command(commands::OUTLINE, &json!({})).map_err(|e| Fail(1, e))?;
    let b = app.book.as_ref().ok_or_else(|| Fail(2, "no book".into()))?;
    let src = b.source.as_ref().map_err(|e| Fail(2, e.clone()))?;
    let words = wordsmith_book_check::verify::book_words(&src.book, true).len();
    // The title and byline as the book prints them: Book's additions over the source's.
    let shown = wordsmith_book_format::assemble::assemble(&b.files.project, &src.book, None).meta;
    let out = json!({
        "title": shown.title, "author": shown.byline, "design": b.files.project.design.id,
        "sections": outline.as_array().map_or(0, Vec::len), "words": words, "metadata": meta,
    });
    println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
    Ok(())
}

fn run(args: &[String]) -> Result<(), Fail> {
    let (_, _, rest) = options(args, &[], &[])?;
    let (project, id, params) = match rest.as_slice() {
        [p, id] => (p, id, json!({})),
        [p, id, json] => (p, id, serde_json::from_str::<Value>(json).map_err(|e| usage(&format!("the parameters are not JSON: {e}")))?),
        _ => return Err(usage("run takes a project, a command id and optional JSON parameters")),
    };
    let mut app = open(project)?;
    let v = app.command(id, &params).map_err(|e| Fail(1, e))?;
    // Commands that change the project save it, as the window's auto-save would.
    if let Some(b) = app.book.as_mut()
        && b.is_dirty()
    {
        b.save().map_err(|e| Fail(1, e))?;
    }
    println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
    Ok(())
}

fn list() {
    for c in commands::list() {
        println!("{:<32} {}", c.id, c.label);
    }
}

fn mcp(args: &[String]) -> Result<(), Fail> {
    let app = match args {
        [] => BookApp::new(),
        [project] => open(project)?,
        _ => return Err(usage("mcp takes at most a project")),
    };
    let mut server = wordsmith_book_ui::agents::server(app, env!("CARGO_PKG_VERSION"));
    server.serve_stdio().map_err(|e| Fail(2, e.to_string()))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((cmd, rest)) = args.split_first() else {
        println!("{USAGE}");
        return ExitCode::from(2);
    };
    let out = match cmd.as_str() {
        "build" => build(rest),
        "check" => check(rest),
        "verify" => verify(rest),
        "convert" => convert(rest),
        "report" => report(rest),
        "run" => run(rest),
        "commands" => {
            list();
            Ok(())
        }
        "mcp" => mcp(rest),
        "help" | "--help" | "-h" => {
            println!("{USAGE}");
            Ok(())
        }
        other => Err(usage(&format!("unknown command “{other}”"))),
    };
    match out {
        Ok(()) => ExitCode::SUCCESS,
        Err(Fail(code, m)) => {
            eprintln!("{m}");
            ExitCode::from(code)
        }
    }
}
