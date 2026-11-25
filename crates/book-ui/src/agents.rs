// SPDX-License-Identifier: MIT OR Apache-2.0
//! Every Book command as an agent tool (AUT-2), served by the core's agent server
//! (`jcj-agents`, the Model Context Protocol over stdio).
//!
//! Book's commands run on a [`BookApp`], not on a document session, so each is registered with
//! the core's engine as a command that runs Book's own dispatch ([`crate::commands::run`]) on
//! the agent's app. Commands of the shell that Book handles (save, undo, redo, open a path) have
//! `book.` names here too ([`ALIASES`]), so every Book action is a `book_…` tool. The app lives
//! on the thread that serves the agent.

use std::cell::RefCell;
use std::sync::Arc;

use jcj_agents::{AgentConfig, AgentInfo, AgentServer};
use jcj_engine::{CmdError, CmdResult, CommandSpec, Session};
use jcj_shell::commands::ids;
use serde_json::Value;

use crate::BookApp;
use crate::commands::{
    ADD_ELEMENT, BACKUP, BLOCK_ONLY, BOX_SET, CLEANUP_APPLY, CLEANUP_KEEP, CLEANUP_LIST, CLEANUP_UNDO, CLOSE, CONTRIBUTOR_ADD, COVER_CHECK,
    COVER_TEMPLATE, DESIGN_CHOOSE, DESIGN_EXPORT, DESIGN_IMPORT, DESIGN_MIX, DESIGN_SAVE_AS, DESIGNS, EBOOK_OPEN, EDITION_ADD, EDITION_REMOVE,
    EDITION_SET, EXPORT, FIX_ADD, FIX_EDIT, FIX_LIST, HINTS, HINTS_SET, HOUSE_JOIN, HOUSE_LEAVE, IMPORT, IMPORT_CONFIRM, IMPORT_REVIEW, ISBN_ASSIGN,
    ISBN_LIST, METADATA, METADATA_SET, MOVE_ELEMENT, NEW, NUMBERING, ONIX, OUTLINE, PREFLIGHT, PREVIEW_PAGE, PREVIEW_SHOW, PROOF, REIMPORT,
    RELOAD_SOURCE, REMOVE_ELEMENT, SAVE_AS_ONE_FILE, SECTION, SERIES_VIEW, SET_ELEMENT, list,
};

thread_local! {
    static APP: RefCell<BookApp> = RefCell::new(BookApp::new());
}

/// Book names for the shell's commands Book handles: (Book's name, the shell's id, label).
pub const ALIASES: [(&str, &str, &str); 4] = [
    ("book.save", ids::SAVE, "Save the Book"),
    ("book.undo", ids::UNDO, "Undo"),
    ("book.redo", ids::REDO, "Redo"),
    ("book.open", ids::OPEN_PATH, "Open a Book"),
];

/// Shell commands that only make sense in the window (they show a dialog or page), with what an
/// agent uses instead.
pub const WINDOW_ONLY: [(&str, &str); 2] = [(ids::OPEN, "book.open"), (ids::NEW_FROM_TEMPLATE, "book.new")];

/// Run a command on the agent's app (this thread's).
pub fn call(id: &str, params: &Value) -> CmdResult {
    let target = ALIASES.iter().find(|(a, _, _)| *a == id).map_or(id, |(_, shell, _)| shell);
    APP.with(|app| app.borrow_mut().command(target, params)).map_err(CmdError::Failed)
}

/// Run something with the agent's app (tests, and the command line before serving).
pub fn with_app<T>(f: impl FnOnce(&mut BookApp) -> T) -> T {
    APP.with(|app| f(&mut app.borrow_mut()))
}

/// What each command takes, for the tool's description.
fn params(id: &str) -> &'static str {
    match id {
        NEW => {
            r#"{"path": folder of the new book, "manuscript": path of a Manuscript project to link (leave out for a book with its own source), "date": "YYYY-MM-DD"}"#
        }
        IMPORT_REVIEW => r#"{"file": path}"#,
        IMPORT => r#"{"file": DOCX, ODT, RTF, Markdown, text or EPUB file, "path": folder of the new book, "off": [import rules to turn off]}"#,
        CLEANUP_APPLY => r#"{"ids": [change ids] (all when left out)}"#,
        CLEANUP_KEEP => r#"{"on": true|false}"#,
        NUMBERING => r#"{"chapters", "parts": label-words|label-figures|words|figures|roman|none, "chapter_label", "part_label"}"#,
        SECTION => r#"{"id": section id, "subtitle", "epigraph", "epigraph_source", "unnumbered": bool, "only": both|print|ebook}"#,
        METADATA_SET => r#"{"field", "value"}"#,
        CONTRIBUTOR_ADD => r#"{"name", "role": author|editor|translator|illustrator|foreword|introduction|cover-designer|narrator}"#,
        EDITION_ADD => r#"{"format": paperback|hardcover|ebook|large-print}"#,
        EDITION_SET => {
            r#"{"id", "isbn", "trim", "paper", "profile", "embed_fonts", "cover", "pdf_standards": bool (print: PDF/A-2b and PDF/UA-1), ...}"#
        }
        EDITION_REMOVE | REMOVE_ELEMENT | DESIGN_CHOOSE => r#"{"id"}"#,
        FIX_ADD => r#"{"section", "find", "replace", "note"}"#,
        FIX_EDIT => r#"{"section", "paragraph": n, "text", "note"}"#,
        DESIGN_MIX => r#"{"part", "from": design id}"#,
        DESIGN_SAVE_AS => r#"{"name"}"#,
        DESIGN_EXPORT | DESIGN_IMPORT | SAVE_AS_ONE_FILE | "book.open" => r#"{"path"}"#,
        ADD_ELEMENT => r#"{"kind": dedication|epigraph|acknowledgements|about-the-author|...}"#,
        MOVE_ELEMENT => r#"{"id", "target": {"element"|"section"|"heading"}, "zone": before|after|inside}"#,
        SET_ELEMENT => r#"{"id", "title", "text"}"#,
        PREVIEW_SHOW => r#"{"section": section id or element:<id>}"#,
        PREVIEW_PAGE => r#"{"page": n}"#,
        EBOOK_OPEN => r#"{"open": false to only write the EPUB}"#,
        PREFLIGHT => r#"{"edition": id}"#,
        EXPORT => r#"{"folder", "editions": [ids], "bundle": bool}"#,
        BLOCK_ONLY => r#"{"section", "starts": the paragraph's first words, "only": print|ebook|both}"#,
        COVER_TEMPLATE => r#"{"edition": a print edition, "folder"}"#,
        COVER_CHECK => r#"{"edition": a print edition, "file": a cover PDF, PNG or JPEG}"#,
        REIMPORT => r#"{"file": the new version, "apply": true to take it}"#,
        HINTS_SET => {
            r#"{"keep": [words and phrases never broken across lines], "hyphenate": ["man-u-script": hyphens where a word may break; none: never]}"#
        }
        _ => "{}",
    }
}

/// Commands that only read.
fn reads_only(id: &str) -> bool {
    matches!(
        id,
        SERIES_VIEW | CLEANUP_LIST | METADATA | FIX_LIST | DESIGNS | OUTLINE | PREVIEW_SHOW | PREVIEW_PAGE | PREFLIGHT | IMPORT_REVIEW | HINTS
    )
}

fn spec(id: &'static str, run: fn(&mut Session, &Value) -> CmdResult) -> CommandSpec {
    let label = list().into_iter().find(|c| c.id == id).map(|c| c.label).or_else(|| ALIASES.iter().find(|a| a.0 == id).map(|a| a.2.to_owned()));
    // Labels live as long as the server: made once per registry.
    let label: &'static str = Box::leak(label.unwrap_or_else(|| id.to_owned()).into_boxed_str());
    let s = CommandSpec::new(id, label, "Book", run).params(params(id));
    if reads_only(id) { s.pure() } else { s }
}

macro_rules! specs {
    ($($id:expr),* $(,)?) => {
        vec![$( spec($id, |_, p| call($id, p)) ),*]
    };
}

/// Every Book command as an engine command.
pub fn command_specs() -> Vec<CommandSpec> {
    specs![
        NEW,
        IMPORT_REVIEW,
        IMPORT_CONFIRM,
        IMPORT,
        CLEANUP_LIST,
        CLEANUP_APPLY,
        CLEANUP_UNDO,
        CLEANUP_KEEP,
        NUMBERING,
        SECTION,
        METADATA,
        METADATA_SET,
        CONTRIBUTOR_ADD,
        EDITION_ADD,
        EDITION_SET,
        EDITION_REMOVE,
        FIX_ADD,
        FIX_EDIT,
        FIX_LIST,
        DESIGNS,
        DESIGN_CHOOSE,
        DESIGN_MIX,
        DESIGN_SAVE_AS,
        DESIGN_EXPORT,
        DESIGN_IMPORT,
        CLOSE,
        SAVE_AS_ONE_FILE,
        BACKUP,
        RELOAD_SOURCE,
        OUTLINE,
        ADD_ELEMENT,
        REMOVE_ELEMENT,
        MOVE_ELEMENT,
        SET_ELEMENT,
        PREVIEW_SHOW,
        PREVIEW_PAGE,
        EBOOK_OPEN,
        PREFLIGHT,
        EXPORT,
        BLOCK_ONLY,
        COVER_TEMPLATE,
        COVER_CHECK,
        REIMPORT,
        HINTS,
        HINTS_SET,
        BOX_SET,
        PROOF,
        HOUSE_JOIN,
        HOUSE_LEAVE,
        SERIES_VIEW,
        ISBN_LIST,
        ISBN_ASSIGN,
        ONIX,
        "book.save",
        "book.undo",
        "book.redo",
        "book.open",
    ]
}

/// What the agent is told when it connects.
pub const INSTRUCTIONS: &str = "Wordsmith Book turns a finished manuscript into a print-ready PDF and an EPUB. \
Start with book_new (link a Manuscript project, or a book with its own source) or book_import (a DOCX, ODT, RTF, Markdown, text or EPUB file), \
or book_open an existing .wsbk folder or .wsbkz file. Then: book_outline and book_metadata to see the book; book_numbering, \
book_section, book_element_add and book_metadata_set to shape it; book_designs and book_design_choose to design it; \
book_edition_set for trim, paper and ISBN; book_preflight to check an edition; book_export to write the PDF, EPUB and bundle \
(nothing is written if a check fails). book_save saves the project. A linked Manuscript project is never changed.";

/// An agent server over the app (which moves to this thread's agent app).
pub fn server(app: BookApp, version: &str) -> AgentServer {
    APP.with(|a| *a.borrow_mut() = app);
    let registry = Arc::new(jcj_engine::registry_with(command_specs()));
    let session = jcj_engine::session_with(jcj_doc::Document::new(), registry);
    let info = AgentInfo::new("wordsmith-book", crate::NAME, version, INSTRUCTIONS);
    AgentServer::headless(session, AgentConfig::new(info).export("book."))
}
