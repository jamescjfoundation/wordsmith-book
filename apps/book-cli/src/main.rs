// SPDX-License-Identifier: MIT OR Apache-2.0
//! Wordsmith Book on the command line (AUT-1): build, check, verify, convert, report, run,
//! commands and mcp. Every command goes through the same code as the window and the agent
//! tools.
//!
//! It follows the core's convention for command lines (core decision 0017): results on
//! standard output, what went wrong on standard error as `wordsmith-book-cli: <reason>`, and the exit
//! codes of [`Exit`]: 0 done; 1 it could not be done (a file could not be read or written, a
//! command refused); 2 the command line was wrong; 3 it ran, and a check of the content did not
//! pass (the words differ, or preflight blocks export).

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use jcj_agents::cli::{Exit, McpArgs, Standard, standard, version_line};
use serde_json::{Value, json};
use wordsmith_book_check::preflight::{Finding, Link, Report};
use wordsmith_book_ui::export::Stopped;
use wordsmith_book_ui::{BookApp, commands};

/// The program's name, as it is run.
const PROGRAM: &str = "wordsmith-book-cli";

fn usage_text() -> String {
    format!(
        "usage: {PROGRAM} <command> [arguments]

  build <project> [--edition <id>]... [--out <folder>] [--bundle]
        Build editions (all when none is named), check them, and write the files.
  check <project> [--edition <id>]...
        Preflight editions (all when none is named); prints what it finds.
  verify <project> <file>
        Check a PDF, EPUB or document against the project's words.
  convert <file> <project folder> [--title <title>]
        Import a DOCX, ODT, RTF, Markdown, text or EPUB file into a new Book project.
  report <project>
        The book's title, author, editions, sections, words and needs, as JSON.
  run <project> <command id> [<JSON parameters>]
        Run any Book command (see `commands`) and print what it gives, as JSON.
  commands
        Every Book command, with its label.
  {}
        Serve Book's commands to an agent as tools (Model Context Protocol over stdio).
        <file> is a Book project to open first; the two folders are the only ones the
        agent's commands may read and write files in.
  --help | --version

Exit codes: 0 done; 1 it could not be done; 2 the command line was wrong;
3 a check did not pass (the words differ, or preflight blocks export).",
        McpArgs::USAGE
    )
}

/// Why a command did not end with a result.
enum Fail {
    /// It could not do what was asked ([`Exit::Failed`]).
    Failed(String),
    /// The command line was wrong ([`Exit::Usage`]).
    Usage(String),
}

/// How a command ended: [`Exit::Done`], or [`Exit::Differs`] with the check's answer already
/// on standard output.
type Outcome = Result<Exit, Fail>;

fn usage(why: impl Into<String>) -> Fail {
    Fail::Usage(why.into())
}

fn failed(why: impl std::fmt::Display) -> Fail {
    Fail::Failed(why.to_string())
}

fn open(path: &str) -> Result<BookApp, Fail> {
    let mut app = BookApp::new();
    app.open_path(path).map_err(|e| failed(format!("{path}: {e}")))?;
    if let Some(Err(e)) = app.book.as_ref().map(|b| &b.source) {
        return Err(failed(format!("{path}: the source could not be read: {e}")));
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
                let v = args.get(i + 1).ok_or_else(|| usage(format!("--{name} needs a value")))?;
                opts.push((name.to_owned(), v.clone()));
                i += 1;
            } else {
                return Err(usage(format!("unknown option --{name}")));
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

/// Print an edition's preflight report: a result, so on standard output.
fn print_report(r: &Report) {
    println!("{} — {}", r.edition, if r.blocked() { "export is blocked" } else { "nothing blocks export" });
    for f in &r.findings {
        println!("  {}", line(f));
    }
}

fn build(args: &[String]) -> Outcome {
    let (opts, flags, rest) = options(args, &["edition", "out"], &["bundle"])?;
    let [project] = rest.as_slice() else { return Err(usage("build takes one project")) };
    let app = open(project)?;
    let b = app.book.as_ref().ok_or_else(|| failed("no book"))?;
    let folder = opts.iter().find(|(k, _)| k == "out").map_or_else(|| wordsmith_book_ui::export::default_folder(b), |(_, out)| PathBuf::from(out));
    let bundle = flags.iter().any(|f| f == "bundle");
    // The same export as the window's and the agents' (`book.export`), with its reason kept.
    match wordsmith_book_ui::export::run_checked(b, &editions(&app, &opts), bundle, &folder, Some(&app.prefs.validator_settings())) {
        Ok(files) => {
            for f in files {
                println!("{}", f.display());
            }
            Ok(Exit::Done)
        }
        // The check ran and said no: its answer is the result, and nothing was written.
        Err(Stopped::Blocked(report)) => {
            print_report(&report);
            println!("{}", Stopped::Blocked(report));
            Ok(Exit::Differs)
        }
        Err(Stopped::Failed(why)) => Err(failed(why)),
    }
}

fn check(args: &[String]) -> Outcome {
    let (opts, _, rest) = options(args, &["edition"], &[])?;
    let [project] = rest.as_slice() else { return Err(usage("check takes one project")) };
    let app = open(project)?;
    let b = app.book.as_ref().ok_or_else(|| failed("no book"))?;
    let validator = app.prefs.validator_settings();
    let mut blocked = false;
    for e in editions(&app, &opts) {
        let r = wordsmith_book_ui::preflight::run(b, &e, &validator).map_err(failed)?;
        print_report(&r);
        blocked |= r.blocked();
    }
    if blocked {
        println!("preflight found problems that block export");
        Ok(Exit::Differs)
    } else {
        Ok(Exit::Done)
    }
}

fn verify(args: &[String]) -> Outcome {
    let (_, _, rest) = options(args, &[], &[])?;
    let [project, file] = rest.as_slice() else { return Err(usage("verify takes a project and a file")) };
    let app = open(project)?;
    let b = app.book.as_ref().ok_or_else(|| failed("no book"))?;
    let src = b.source.as_ref().map_err(failed)?;
    let book = wordsmith_book_format::fixes::book_for_build(&b.files.project.fixes, src);
    let bytes = std::fs::read(file).map_err(|e| failed(format!("{file}: {e}")))?;
    let ext = Path::new(file).extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    let source_words = wordsmith_book_check::verify::book_words(&book, true);
    // What the check found is its answer: on standard output, with 0 for "the same" and 3 for
    // "they differ".
    let compare = |found: &[String]| match wordsmith_book_check::verify::compare(&source_words, found) {
        Ok(n) => {
            println!("every word matches ({n} words)");
            Exit::Done
        }
        Err(d) => {
            println!("{file}: {d}");
            Exit::Differs
        }
    };
    match ext.as_str() {
        "epub" => Ok(compare(&wordsmith_book_check::epub::words(&bytes, true).map_err(failed)?)),
        "pdf" => {
            // A PDF is checked against this project's own layout of its print edition.
            let edition = b.files.project.editions.iter().find(|e| e.format.is_print()).ok_or_else(|| failed("the book has no print edition"))?;
            let print = wordsmith_book_check::build::print(&b.files.project, &b.files.designs, &book, edition, &|p| b.picture(p)).map_err(failed)?;
            let found = wordsmith_book_check::preflight::print_words(&print, &bytes, &book);
            Ok(match found.first() {
                None => {
                    println!("every letter matches");
                    Exit::Done
                }
                Some(f) => {
                    println!("{file}: {}", f.message);
                    Exit::Differs
                }
            })
        }
        _ => {
            let format = match ext.as_str() {
                "docx" => wordsmith_book_import::Format::Docx,
                "odt" => wordsmith_book_import::Format::Odt,
                "rtf" => wordsmith_book_import::Format::Rtf,
                "md" | "markdown" => wordsmith_book_import::Format::Markdown,
                "txt" => wordsmith_book_import::Format::Text,
                // The command line is right; this file is one Book cannot check.
                _ => return Err(failed(format!("{file}: Book reads PDF, EPUB, DOCX, ODT, RTF, Markdown and text files"))),
            };
            // A document is read as Book imports it, so its words compare with the book's.
            let i = wordsmith_book_import::import(format, &bytes, &b.files.project.cleanup.rules, "").map_err(failed)?;
            Ok(compare(&wordsmith_book_check::verify::book_words(&wordsmith_manuscript_format::book::from_project(&i.project), true)))
        }
    }
}

fn convert(args: &[String]) -> Outcome {
    let (opts, _, rest) = options(args, &["title"], &[])?;
    let [file, project] = rest.as_slice() else { return Err(usage("convert takes a file and a project folder")) };
    if Path::new(project).exists() {
        return Err(failed(format!("{project} already exists")));
    }
    let mut app = BookApp::new();
    let v = app.command(commands::IMPORT, &json!({ "file": file, "path": project })).map_err(failed)?;
    if let Some((_, t)) = opts.iter().find(|(k, _)| k == "title") {
        app.command(commands::METADATA_SET, &json!({ "field": "title", "value": t })).map_err(failed)?;
        app.book.as_mut().map(|b| b.save()).transpose().map_err(failed)?;
    }
    println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
    Ok(Exit::Done)
}

fn report(args: &[String]) -> Outcome {
    let (_, _, rest) = options(args, &[], &[])?;
    let [project] = rest.as_slice() else { return Err(usage("report takes one project")) };
    let mut app = open(project)?;
    let meta = app.command(commands::METADATA, &json!({})).map_err(failed)?;
    let outline = app.command(commands::OUTLINE, &json!({})).map_err(failed)?;
    let b = app.book.as_ref().ok_or_else(|| failed("no book"))?;
    let src = b.source.as_ref().map_err(failed)?;
    let words = wordsmith_book_check::verify::book_words(&src.book, true).len();
    // The title and byline as the book prints them: Book's additions over the source's.
    let shown = wordsmith_book_format::assemble::assemble(&b.files.project, &src.book, None).meta;
    let out = json!({
        "title": shown.title, "author": shown.byline, "design": b.files.project.design.id,
        "sections": outline.as_array().map_or(0, Vec::len), "words": words, "metadata": meta,
    });
    println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
    Ok(Exit::Done)
}

fn run(args: &[String]) -> Outcome {
    let (_, _, rest) = options(args, &[], &[])?;
    let (project, id, params) = match rest.as_slice() {
        [p, id] => (p, id, json!({})),
        [p, id, json] => (p, id, serde_json::from_str::<Value>(json).map_err(|e| usage(format!("the parameters are not JSON: {e}")))?),
        _ => return Err(usage("run takes a project, a command id and optional JSON parameters")),
    };
    let mut app = open(project)?;
    let v = app.command(id, &params).map_err(failed)?;
    // Commands that change the project save it, as the window's auto-save would.
    if let Some(b) = app.book.as_mut()
        && b.is_dirty()
    {
        b.save().map_err(failed)?;
    }
    println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
    Ok(Exit::Done)
}

fn list(args: &[String]) -> Outcome {
    if let Some(extra) = args.first() {
        return Err(usage(format!("commands takes no argument, and {extra} is one")));
    }
    for c in commands::list() {
        println!("{:<32} {}", c.id, c.label);
    }
    Ok(Exit::Done)
}

/// `mcp [<file>] [--read-root <folder>] [--write-root <folder>]`: serve Book's commands as
/// tools until standard input ends. Standard output carries the protocol and nothing else.
fn mcp(args: &[String]) -> Outcome {
    let args = McpArgs::parse(args).map_err(usage)?;
    // A folder that does not exist is a wrong command line: said before anything is opened.
    args.roots().map_err(usage)?;
    // The project named here is the person's choice, not the agent's: it is opened before the
    // folders are put in force.
    let app = match &args.file {
        Some(project) => open(project)?,
        None => BookApp::new(),
    };
    let roots = args.confine(PROGRAM).map_err(usage)?;
    wordsmith_book_ui::agents::server(app, env!("CARGO_PKG_VERSION"), roots).serve_stdio().map_err(|e| failed(format!("mcp: {e}")))?;
    Ok(Exit::Done)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rest = args.get(1..).unwrap_or(&[]);
    let outcome = match (standard(&args), args.first().map(String::as_str)) {
        (Some(Standard::Help), _) => {
            println!("{}", usage_text());
            Ok(Exit::Done)
        }
        (Some(Standard::Version), _) => {
            println!("{}", version_line(PROGRAM, env!("CARGO_PKG_VERSION")));
            Ok(Exit::Done)
        }
        // Nothing was asked, so nothing was done: a caller that lost its arguments must not
        // read success.
        (Some(Standard::NoCommand), _) | (None, None) => Err(usage("no command was given")),
        (None, Some("build")) => build(rest),
        (None, Some("check")) => check(rest),
        (None, Some("verify")) => verify(rest),
        (None, Some("convert")) => convert(rest),
        (None, Some("report")) => report(rest),
        (None, Some("run")) => run(rest),
        (None, Some("commands")) => list(rest),
        (None, Some("mcp")) => mcp(rest),
        (None, Some(option)) if option.starts_with('-') => Err(usage(format!("unknown option {option}"))),
        (None, Some(other)) => Err(usage(format!("unknown command “{other}”"))),
    };
    match outcome {
        Ok(exit) => exit.into(),
        // Results go to standard output; what went wrong, to standard error.
        Err(Fail::Failed(why)) => {
            eprintln!("{PROGRAM}: {why}");
            Exit::Failed.into()
        }
        Err(Fail::Usage(why)) => {
            eprintln!("{PROGRAM}: {why}");
            eprintln!("{}", usage_text());
            Exit::Usage.into()
        }
    }
}
