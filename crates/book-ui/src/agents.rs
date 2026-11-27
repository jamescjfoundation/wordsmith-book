// SPDX-License-Identifier: MIT OR Apache-2.0
//! Every Book command as an agent tool (AUT-2), through the core's `jcj-agents` (the Model
//! Context Protocol over stdio).
//!
//! Book's commands run on a [`BookApp`] (a project), not on a document session, so they are
//! given to the core's server as an app's own commands ([`AppCommands`]): the server names a
//! tool `book_<command>` after each, lists it, checks the files its parameters name against the
//! folders the agent may read and write, and calls it. No general document tools stand beside
//! them: there is no document behind them here. Commands of the shell that Book handles (save,
//! undo, redo, open a path) have `book.` names too ([`ALIASES`]), so every Book action is a
//! `book_…` tool.
//!
//! While folders are in force (`wordsmith-book-cli mcp --read-root … --write-root …`):
//!
//! - which parameters of a command name files it reads and files it writes is said in one
//!   place, [`files_of`], and the server checks them;
//! - what a command reads or writes without being told where (the export folder beside the
//!   project, a new book beside the imported file) is checked here ([`BookTools`]);
//! - the project named on the command line is the person's choice: it is read and saved where
//!   it lives. A project the agent opens or makes is the agent's: it is read inside the read
//!   folder, with the manuscript it links, and changed only inside the write folder;
//! - a command that starts another program is refused: a program cannot be kept to a folder.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use jcj_agents::{AgentConfig, AgentInfo, AgentServer, AppCommand, AppCommands};
use jcj_engine::file_access::{self, AutomationRoots};
use jcj_shell::commands::ids;
use serde_json::Value;

use crate::BookApp;
use crate::commands::{
    BACKUP, BOX_SET, CLEANUP_LIST, CLOSE, CONTRIBUTOR_ADD, COVER_CHECK, COVER_TEMPLATE, DESIGN_CHOOSE, DESIGN_EXPORT, DESIGN_IMPORT, DESIGN_MIX,
    DESIGN_SAVE_AS, DESIGNS, EBOOK_OPEN, EDITION_ADD, EDITION_REMOVE, EDITION_SET, EXPORT, FIX_ADD, FIX_EDIT, FIX_LIST, HINTS, HINTS_SET, HOUSE_JOIN,
    IMPORT, IMPORT_CONFIRM, IMPORT_REVIEW, ISBN_ASSIGN, ISBN_LIST, METADATA, METADATA_SET, MOVE_ELEMENT, NEW, NUMBERING, ONIX, OUTLINE, PREFLIGHT,
    PREVIEW_PAGE, PREVIEW_SHOW, PROOF, REIMPORT, RELOAD_SOURCE, REMOVE_ELEMENT, SAVE_AS_ONE_FILE, SECTION, SERIES_VIEW, SET_ELEMENT, list,
};

/// The app's short name: every tool's name starts with it and an underscore.
pub const PREFIX: &str = "book";

/// Open a Book by its path, as an agent names it (the shell's id is the window's).
pub const OPEN: &str = "book.open";

/// Book names for the shell's commands Book handles: (Book's name, the shell's id, label).
pub const ALIASES: [(&str, &str, &str); 4] = [
    ("book.save", ids::SAVE, "Save the Book"),
    ("book.undo", ids::UNDO, "Undo"),
    ("book.redo", ids::REDO, "Redo"),
    (OPEN, ids::OPEN_PATH, "Open a Book"),
];

/// Shell commands that only make sense in the window (they show a dialog or page), with what an
/// agent uses instead.
pub const WINDOW_ONLY: [(&str, &str); 2] = [(ids::OPEN, OPEN), (ids::NEW_FROM_TEMPLATE, "book.new")];

/// What each command takes, for the tool's description.
fn params(id: &str) -> &'static str {
    match id {
        NEW => {
            r#"{"path": folder of the new book, "manuscript": path of a Manuscript project to link (leave out for a book with its own source), "date": "YYYY-MM-DD"}"#
        }
        IMPORT_REVIEW => r#"{"file": path}"#,
        IMPORT => r#"{"file": DOCX, ODT, RTF, Markdown, text or EPUB file, "path": folder of the new book, "off": [import rules to turn off]}"#,
        crate::commands::CLEANUP_APPLY => r#"{"ids": [change ids] (all when left out)}"#,
        crate::commands::CLEANUP_KEEP => r#"{"on": true|false}"#,
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
        DESIGN_EXPORT | DESIGN_IMPORT | SAVE_AS_ONE_FILE | OPEN => r#"{"path"}"#,
        crate::commands::ADD_ELEMENT => r#"{"kind": dedication|epigraph|acknowledgements|about-the-author|...}"#,
        MOVE_ELEMENT => r#"{"id", "target": {"element"|"section"|"heading"}, "zone": before|after|inside}"#,
        SET_ELEMENT => r#"{"id", "title", "text"}"#,
        PREVIEW_SHOW => r#"{"section": section id or element:<id>}"#,
        PREVIEW_PAGE => r#"{"page": n}"#,
        EBOOK_OPEN => r#"{"open": false to only write the EPUB}"#,
        PREFLIGHT => r#"{"edition": id}"#,
        EXPORT => r#"{"folder", "editions": [ids], "bundle": bool}"#,
        crate::commands::BLOCK_ONLY => r#"{"section", "starts": the paragraph's first words, "only": print|ebook|both}"#,
        COVER_TEMPLATE => r#"{"edition": a print edition, "folder"}"#,
        COVER_CHECK => r#"{"edition": a print edition, "file": a cover PDF, PNG or JPEG}"#,
        REIMPORT => r#"{"file": the new version, "apply": true to take it}"#,
        HINTS_SET => {
            r#"{"keep": [words and phrases never broken across lines], "hyphenate": ["man-u-script": hyphens where a word may break; none: never]}"#
        }
        _ => "{}",
    }
}

/// Commands that only read: nothing changes, in the book or on disk.
fn reads_only(id: &str) -> bool {
    matches!(
        id,
        SERIES_VIEW
            | CLEANUP_LIST
            | METADATA
            | FIX_LIST
            | DESIGNS
            | OUTLINE
            | PREVIEW_SHOW
            | PREVIEW_PAGE
            | PREFLIGHT
            | IMPORT_REVIEW
            | HINTS
            | ISBN_LIST
            | COVER_CHECK
    )
}

/// The parameters of command `id` that name files or folders: those it reads, then those it
/// writes. Every other command names none. `book.new`'s `manuscript` and a box set's are not
/// here, because Book reads them from the book's folder, not from where the program runs:
/// [`BookTools`] checks them where Book will read them.
pub fn files_of(id: &str) -> (&'static [&'static str], &'static [&'static str]) {
    match id {
        NEW | DESIGN_EXPORT | SAVE_AS_ONE_FILE => (&[], &["path"]),
        OPEN | DESIGN_IMPORT => (&["path"], &[]),
        IMPORT => (&["file"], &["path"]),
        IMPORT_REVIEW | COVER_CHECK | REIMPORT => (&["file"], &[]),
        EXPORT | ONIX | PROOF | COVER_TEMPLATE => (&[], &["folder"]),
        HOUSE_JOIN | ISBN_LIST | SERIES_VIEW => (&["house"], &[]),
        // Taking an ISBN records its use in the house file.
        ISBN_ASSIGN => (&["house"], &["house"]),
        _ => (&[], &[]),
    }
}

/// Commands that open or make a book in place of the open one.
fn replaces_book(id: &str) -> bool {
    matches!(id, NEW | IMPORT | IMPORT_REVIEW | IMPORT_CONFIRM | OPEN)
}

/// Commands that change the open book, in memory or in its folder. The rest only read, open or
/// make another book, or write files somewhere else (an export, a cover template).
fn changes_book(id: &str) -> bool {
    !(reads_only(id)
        || replaces_book(id)
        || matches!(id, CLOSE | RELOAD_SOURCE | EXPORT | ONIX | PROOF | COVER_TEMPLATE | SAVE_AS_ONE_FILE | DESIGN_EXPORT | EBOOK_OPEN))
}

/// Every Book command as the agent sees it, in the order of Book's command list.
pub fn commands() -> Vec<AppCommand> {
    list()
        .into_iter()
        .filter_map(|c| {
            let (id, label) = if c.id.starts_with("book.") {
                (c.id.clone(), c.label.clone())
            } else {
                let (alias, _, label) = ALIASES.iter().find(|(_, shell, _)| *shell == c.id)?;
                ((*alias).to_owned(), (*label).to_owned())
            };
            let (reads, writes) = files_of(&id);
            let mut command = AppCommand::new(id.as_str(), label).params(params(&id)).reads(reads).writes(writes);
            command.read_only = reads_only(&id);
            Some(command)
        })
        .collect()
}

/// The names of the tools, in the order the server lists them.
pub fn tool_names() -> Vec<String> {
    commands().iter().map(|c| jcj_agents::app_tool_name(PREFIX, &c.id)).collect()
}

/// What the agent is told when it connects.
pub const INSTRUCTIONS: &str = "Wordsmith Book turns a finished manuscript into a print-ready PDF and an EPUB. \
Start with book_new (link a Manuscript project, or a book with its own source) or book_import (a DOCX, ODT, RTF, Markdown, text or EPUB file), \
or book_open an existing .wsbk folder or .wsbkz file. Then: book_outline and book_metadata to see the book; book_numbering, \
book_section, book_element_add and book_metadata_set to shape it; book_designs and book_design_choose to design it; \
book_edition_set for trim, paper and ISBN; book_preflight to check an edition; book_export to write the PDF, EPUB and bundle \
(nothing is written if a check fails). book_save saves the project. A linked Manuscript project is never changed.";

/// Book's commands on one app, for the core's server.
pub struct BookTools {
    app: BookApp,
    /// The folder of the project the server was started on: the person's choice.
    own: Option<PathBuf>,
}

fn text<'a>(params: &'a Value, key: &str) -> Option<&'a str> {
    params.get(key).and_then(Value::as_str)
}

/// `link` as Book reads it: from `folder`, with each `..` taking one folder off. The folder of a
/// book being made does not exist yet, and a `..` below a missing folder cannot be checked, so
/// the path is put in order here first.
fn from_folder(folder: &Path, link: &str) -> PathBuf {
    let mut out = PathBuf::new();
    for part in folder.join(link).components() {
        match part {
            std::path::Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            std::path::Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}

fn may_read(path: &Path) -> Result<(), String> {
    file_access::check_read(&path.to_string_lossy())
}

fn may_write(path: &Path) -> Result<(), String> {
    file_access::check_write(&path.to_string_lossy())
}

impl BookTools {
    /// The tools over `app`. A book it has open is the person's (see the module's rules).
    pub fn new(app: BookApp) -> Self {
        let own = app.book.as_ref().map(|b| b.root.clone());
        BookTools { app, own }
    }

    /// Whether the open book is the one the server was started on.
    fn on_own_book(&self) -> bool {
        self.app.book.as_ref().is_some_and(|b| self.own.as_deref() == Some(b.root.as_path()))
    }

    /// The checks the server cannot make from the parameters alone, while folders are in force:
    /// what `id` would read or write without a parameter that says where.
    fn check(&self, id: &str, params: &Value) -> Result<(), String> {
        let book = self.app.book.as_ref();
        match id {
            // A linked manuscript is named from the new book's folder.
            NEW => {
                if let (Some(path), Some(manuscript)) = (text(params, "path"), text(params, "manuscript")) {
                    may_read(&from_folder(Path::new(path), manuscript))?;
                }
            }
            // A packed project is unpacked beside itself the first time it is opened.
            OPEN => {
                let path = Path::new(text(params, "path").unwrap_or(""));
                if path.extension().is_some_and(|e| e == wordsmith_book_format::PACKED_EXTENSION) {
                    let folder = path.with_extension(wordsmith_book_format::EXTENSION);
                    if !folder.join("book.toml").exists() {
                        may_write(&folder)?;
                    }
                }
            }
            // Without `path` the new book is made beside the file.
            IMPORT => {
                if let (None, Some(file)) = (text(params, "path"), text(params, "file")) {
                    may_write(&Path::new(file).with_extension(wordsmith_book_format::EXTENSION))?;
                }
            }
            IMPORT_CONFIRM => {
                if let Some(review) = &self.app.review {
                    may_write(Path::new(&review.target))?;
                }
            }
            // Without `folder` these write into the export folder beside the project.
            EXPORT | ONIX | PROOF => {
                if let (None, Some(b)) = (text(params, "folder"), book) {
                    may_write(&crate::export::default_folder(b))?;
                }
            }
            EBOOK_OPEN => {
                if params.get("open").and_then(Value::as_bool).unwrap_or(true) {
                    return Err(
                        "this starts a reading app, which cannot be kept to the folders this agent was given; pass \"open\": false to only write the EPUB"
                            .into(),
                    );
                }
                may_write(&crate::commands::ebook_preview_folder())?;
            }
            // Each book of a box set is named from the project's folder.
            BOX_SET => {
                let books = params.get("books").and_then(Value::as_array).into_iter().flatten();
                for manuscript in books.filter_map(|b| text(b, "manuscript")) {
                    if let Some(b) = book {
                        may_read(&from_folder(&b.root, manuscript))?;
                    }
                }
            }
            _ => {}
        }
        // The agent's own book is changed only inside the write folder: refused here, before
        // anything changes in memory, and not at the save that would follow.
        if let Some(b) = book.filter(|_| changes_book(id) && !self.on_own_book()) {
            may_write(&b.root)?;
            if id == BACKUP {
                may_write(&b.backup_dir())?;
            }
            // A fix to a linked manuscript is offered in a file beside the writer's project.
            if matches!(id, FIX_ADD | FIX_EDIT)
                && let Ok(source) = &b.source
                && source.kind != wordsmith_book_format::source::Kind::Owned
            {
                may_write(&wordsmith_manuscript_format::offers::offers_path(&source.path))?;
            }
        }
        Ok(())
    }

    /// The manuscripts the agent's book links are read where its project says, which no
    /// parameter names: a book that links one outside the read folder is not kept open.
    fn keep_to_linked_sources(&mut self) -> Result<(), String> {
        let Some(b) = self.app.book.as_ref().filter(|_| !self.on_own_book()) else { return Ok(()) };
        let project = &b.files.project;
        let set = wordsmith_book_format::boxset::books(project);
        let sources = if set.is_empty() { vec![project.source.clone()] } else { set.into_iter().map(|(_, source)| source).collect() };
        for source in sources {
            if let wordsmith_book_model::Source::Manuscript(link) = source
                && let Err(why) = may_read(&from_folder(&b.root, &link))
            {
                self.app.book = None;
                return Err(format!("the book was not kept open, because it links a manuscript this agent may not read: {why}"));
            }
        }
        Ok(())
    }
}

impl AppCommands for BookTools {
    fn commands(&self) -> Vec<AppCommand> {
        commands()
    }

    fn run(&mut self, id: &str, params: &Value) -> Result<Value, String> {
        let target = ALIASES.iter().find(|(alias, _, _)| *alias == id).map_or(id, |(_, shell, _)| shell);
        if file_access::current().is_none() {
            return self.app.command(target, params);
        }
        // The server has checked the files the parameters name ([`files_of`]).
        self.check(id, params)?;
        if self.on_own_book() && !replaces_book(id) {
            // The person's project is read and saved where it lives, whatever the folders.
            file_access::unconfined(|| self.app.command(target, params))
        } else {
            // Everything else stays under the folders: the core's file reading and writing
            // refuses a path outside them, wherever Book's code got it from.
            let out = self.app.command(target, params)?;
            self.keep_to_linked_sources()?;
            Ok(out)
        }
    }
}

/// An agent server over the app: every Book command, and nothing else. `roots` are the folders
/// the agent's commands may read and write (`None`: nothing is confined).
pub fn server(app: BookApp, version: &str, roots: Option<Arc<AutomationRoots>>) -> AgentServer {
    let info = AgentInfo::new("wordsmith-book", crate::NAME, version, INSTRUCTIONS);
    AgentServer::for_app(Box::new(BookTools::new(app)), PREFIX, AgentConfig::new(info).confined(roots))
}
