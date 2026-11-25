// SPDX-License-Identifier: MIT OR Apache-2.0
//! Book's commands: ids, labels, keys, and what each does. Every change goes through here, so
//! the window, keys, agents (AUT-2) and tests do the same thing.

use std::path::PathBuf;

use jcj_shell::CommandInfo;
use jcj_shell::commands::ids;
use jcj_shell::tree::Zone;
use serde_json::{Value, json};
use wordsmith_book_model::{Element, ElementKind, Placement, SectionId, Source};

use crate::BookApp;
use crate::doc::OpenBook;
use crate::outline::{self, Key};

/// Make a new book: `{"path": folder, "manuscript": path of a Manuscript project}` links it;
/// without `manuscript` the book gets an empty source of its own.
pub const NEW: &str = "book.new";
/// Review a file before importing it: `{"file": path}`.
pub const IMPORT_REVIEW: &str = "book.importReview";
/// Import the reviewed file with the review's rules.
pub const IMPORT_CONFIRM: &str = "book.importConfirm";
/// Import a file straight into a new book: `{"file", "path", "off": [rule names]}`; gives the
/// report.
pub const IMPORT: &str = "book.import";
/// The cleanup changes proposed for the book's own source.
pub const CLEANUP_LIST: &str = "book.cleanup.list";
/// Make cleanup changes: `{"ids": [..]}`, or all of them; gives how many were made.
pub const CLEANUP_APPLY: &str = "book.cleanup.apply";
/// Put the source back as it was before the last cleanup.
pub const CLEANUP_UNDO: &str = "book.cleanup.undo";
/// "Keep exactly as written": `{"on": true}` proposes no cleanup.
pub const CLEANUP_KEEP: &str = "book.cleanup.keepExactly";
/// Set chapter and part numbering: `{"chapters", "parts", "chapter_label", "part_label"}`
/// (styles: label-words, label-figures, words, figures, roman, none).
pub const NUMBERING: &str = "book.numbering";
/// Set Book's settings for a section: `{"id", "subtitle", "epigraph", "epigraph_source",
/// "unnumbered", "only"}`.
pub const SECTION: &str = "book.section";
/// The book's metadata, editions and needs.
pub const METADATA: &str = "book.metadata";
/// Set a metadata field Book keeps: `{"field", "value"}`.
pub const METADATA_SET: &str = "book.metadata.set";
/// Add a contributor: `{"name", "role"}`.
pub const CONTRIBUTOR_ADD: &str = "book.contributor.add";
/// Add an edition: `{"format"}` (paperback, hardcover, ebook, large-print); gives its id.
pub const EDITION_ADD: &str = "book.edition.add";
/// Change an edition: `{"id", "isbn", "trim", "paper", ...}`.
pub const EDITION_SET: &str = "book.edition.set";
/// Remove an edition: `{"id"}`.
pub const EDITION_REMOVE: &str = "book.edition.remove";
/// Make a light fix: `{"section", "find", "replace", "note"}`; gives its id.
pub const FIX_ADD: &str = "book.fix.add";
/// Make a light fix from an edited paragraph: `{"section", "paragraph": n, "text", "note"}`.
pub const FIX_EDIT: &str = "book.fix.edit";
/// The fixes and where each stands.
pub const FIX_LIST: &str = "book.fix.list";
/// The designs the book can use.
pub const DESIGNS: &str = "book.designs";
/// Choose a design: `{"id"}`.
pub const DESIGN_CHOOSE: &str = "book.design.choose";
/// Take a part from another design: `{"part", "from"}` (no `from`: the book's design's own).
pub const DESIGN_MIX: &str = "book.design.mix";
/// Save the book's design as the user's own: `{"name"}`; gives its id.
pub const DESIGN_SAVE_AS: &str = "book.design.saveAs";
/// Export the book's design as one file: `{"path"}`.
pub const DESIGN_EXPORT: &str = "book.design.export";
/// Import a design file: `{"path"}`; gives its id.
pub const DESIGN_IMPORT: &str = "book.design.import";
/// Close the book.
pub const CLOSE: &str = "book.close";
/// Save the whole project as one packed file: `{"path": file}`.
pub const SAVE_AS_ONE_FILE: &str = "book.saveAsOneFile";
/// Write a packed backup now.
pub const BACKUP: &str = "book.backup";
/// Read the source again.
pub const RELOAD_SOURCE: &str = "book.reloadSource";
/// The outline as data.
pub const OUTLINE: &str = "book.outline";
/// Add an element of Book's own: `{"kind": "dedication"}`; gives its id.
pub const ADD_ELEMENT: &str = "book.element.add";
/// Remove an element: `{"id": id}`.
pub const REMOVE_ELEMENT: &str = "book.element.remove";
/// Move an element: `{"id", "target": {"element"|"section"|"heading"}, "zone": "before"|"after"|"inside"}`.
pub const MOVE_ELEMENT: &str = "book.element.move";
/// Change an element's title, text or outputs: `{"id", "title"?, "text"?, "only"?, "stores"?:
/// [store names] (shown only in editions for those stores)}`.
pub const SET_ELEMENT: &str = "book.element.set";

/// Build the print preview and go to a section's first page: `{"section": id}` (a section's id
/// or `element:<id>`); gives the page (from 1).
pub const PREVIEW_SHOW: &str = "book.preview.show";
/// Which section a page of the print preview belongs to: `{"page": n}` (from 1); gives its id.
pub const PREVIEW_PAGE: &str = "book.preview.page";

/// Build the ebook and hand it to the reading app the user has installed: `{"open": false}`
/// only writes it; gives its path.
pub const EBOOK_OPEN: &str = "book.ebook.openInReadingApp";

/// Build an edition and check it: `{"edition": id}` (the first edition when left out); gives
/// whether export is blocked and every finding with its link.
pub const PREFLIGHT: &str = "book.preflight";

/// Export: `{"folder": path, "editions": [ids] (all when left out), "bundle": bool}`; each
/// edition is built, word-checked and preflighted first and nothing is written if anything
/// blocks; gives the files written.
pub const EXPORT: &str = "book.export";

/// Show a paragraph in one kind of output only: `{"section", "starts": its first words, "only":
/// "print" | "ebook" | "both"}` (both removes the rule). The words must start exactly one
/// paragraph of the section.
pub const BLOCK_ONLY: &str = "book.block.only";

/// Every command Book has, with its label and key.
pub fn list() -> Vec<CommandInfo> {
    let p = |mut c: CommandInfo| {
        c.takes_params = true;
        c
    };
    vec![
        CommandInfo { icon: "save".into(), ..CommandInfo::new(ids::SAVE, "Save", "Mod+S") },
        CommandInfo { icon: "undo".into(), ..CommandInfo::new(ids::UNDO, "Undo", "Mod+Z") },
        CommandInfo { icon: "redo".into(), ..CommandInfo::new(ids::REDO, "Redo", "Mod+Shift+Z") },
        CommandInfo::new(ids::OPEN, "Open…", "Mod+O"),
        p(CommandInfo::new(ids::OPEN_PATH, "Open a Book", "")),
        p(CommandInfo::new(ids::NEW_FROM_TEMPLATE, "New Book from Template", "")),
        p(CommandInfo::new(NEW, "New Book", "Mod+N")),
        CommandInfo::new(CLOSE, "Close Book", "Mod+W"),
        p(CommandInfo::new(IMPORT_REVIEW, "Import a File…", "Mod+Shift+I")),
        CommandInfo::new(IMPORT_CONFIRM, "Import", ""),
        p(CommandInfo::new(IMPORT, "Import a File into a New Book", "")),
        CommandInfo::new(CLEANUP_LIST, "Cleanup Changes", ""),
        p(CommandInfo::new(NUMBERING, "Chapter Numbers", "")),
        CommandInfo::new(METADATA, "Book Details", ""),
        CommandInfo::new(DESIGNS, "Designs", ""),
        p(CommandInfo::new(DESIGN_CHOOSE, "Choose a Design", "")),
        p(CommandInfo::new(DESIGN_MIX, "Take a Part from Another Design", "")),
        p(CommandInfo::new(DESIGN_SAVE_AS, "Save Design as My Own", "")),
        p(CommandInfo::new(DESIGN_EXPORT, "Export Design", "")),
        p(CommandInfo::new(DESIGN_IMPORT, "Import Design", "")),
        p(CommandInfo::new(FIX_ADD, "Fix Words", "")),
        p(CommandInfo::new(FIX_EDIT, "Fix a Paragraph", "")),
        CommandInfo::new(FIX_LIST, "Fixes", ""),
        p(CommandInfo::new(METADATA_SET, "Set a Book Detail", "")),
        p(CommandInfo::new(CONTRIBUTOR_ADD, "Add a Contributor", "")),
        p(CommandInfo::new(EDITION_ADD, "Add an Edition", "")),
        p(CommandInfo::new(EDITION_SET, "Change an Edition", "")),
        p(CommandInfo::new(EDITION_REMOVE, "Remove an Edition", "")),
        p(CommandInfo::new(SECTION, "Chapter Settings", "")),
        p(CommandInfo::new(CLEANUP_APPLY, "Make Cleanup Changes", "")),
        CommandInfo::new(CLEANUP_UNDO, "Undo Last Cleanup", ""),
        p(CommandInfo::new(CLEANUP_KEEP, "Keep Exactly as Written", "")),
        p(CommandInfo::new(SAVE_AS_ONE_FILE, "Save as One File", "")),
        CommandInfo::new(BACKUP, "Back Up Now", ""),
        CommandInfo::new(RELOAD_SOURCE, "Read the Manuscript Again", ""),
        CommandInfo::new(OUTLINE, "Outline", ""),
        p(CommandInfo::new(ADD_ELEMENT, "Add Front or Back Matter", "")),
        p(CommandInfo::new(REMOVE_ELEMENT, "Remove Element", "")),
        p(CommandInfo::new(MOVE_ELEMENT, "Move Element", "")),
        p(CommandInfo::new(SET_ELEMENT, "Edit Element", "")),
        p(CommandInfo::new(PREVIEW_SHOW, "Show in Print Preview", "")),
        p(CommandInfo::new(PREVIEW_PAGE, "Section of a Preview Page", "")),
        p(CommandInfo::new(EBOOK_OPEN, "Open in a Reading App", "")),
        p(CommandInfo::new(PREFLIGHT, "Preflight", "")),
        p(CommandInfo::new(EXPORT, "Export", "")),
        p(CommandInfo::new(BLOCK_ONLY, "Show a Paragraph in Print or Ebook Only", "")),
    ]
}

/// A key of the outline as JSON.
pub fn key_json(k: &Key) -> Value {
    match k {
        Key::Element(id) => json!({ "element": id }),
        Key::Section(id) => json!({ "section": id.as_text() }),
        Key::Heading(Placement::Front, false) => json!({ "heading": "front" }),
        Key::Heading(_, true) => json!({ "heading": "body" }),
        Key::Heading(Placement::Back, false) => json!({ "heading": "back" }),
    }
}

/// A key of the outline from JSON.
pub fn key_from(v: &Value) -> Result<Key, String> {
    if let Some(id) = v.get("element").and_then(Value::as_str) {
        return Ok(Key::Element(id.to_owned()));
    }
    if let Some(id) = v.get("section").and_then(Value::as_str) {
        return id.parse::<SectionId>().map(Key::Section).map_err(|e| e.to_string());
    }
    match v.get("heading").and_then(Value::as_str) {
        Some("front") => Ok(Key::Heading(Placement::Front, false)),
        Some("body") => Ok(Key::Heading(Placement::Front, true)),
        Some("back") => Ok(Key::Heading(Placement::Back, false)),
        _ => Err("a target is an element, a section or a heading".into()),
    }
}

/// A drop zone's name.
pub fn zone_name(z: Zone) -> &'static str {
    match z {
        Zone::Before => "before",
        Zone::After => "after",
        Zone::Inside => "inside",
    }
}

fn zone_from(s: &str) -> Result<Zone, String> {
    match s {
        "before" => Ok(Zone::Before),
        "after" => Ok(Zone::After),
        "inside" => Ok(Zone::Inside),
        _ => Err(format!("`{s}` is not a zone (before, after, inside)")),
    }
}

fn s<'a>(params: &'a Value, key: &str) -> Result<&'a str, String> {
    params.get(key).and_then(Value::as_str).ok_or_else(|| format!("`{key}` is needed"))
}

/// An element kind from its key (`dedication`, `about-the-author`, ...).
pub fn kind_from(key: &str) -> Result<ElementKind, String> {
    serde_json::from_value(Value::String(key.to_owned())).map_err(|_| format!("`{key}` is not a kind of front or back matter"))
}

fn new_book(app: &mut BookApp, params: &Value) -> Result<Value, String> {
    let path = PathBuf::from(s(params, "path")?);
    let date = params.get("date").and_then(Value::as_str).map_or_else(crate::today, str::to_owned);
    let source = match params.get("manuscript").and_then(Value::as_str) {
        Some(m) => {
            let target = PathBuf::from(m);
            let rel = if target.is_absolute() { wordsmith_manuscript_model::series::relative(&path, &target) } else { m.to_owned() };
            Source::Manuscript(rel)
        }
        None => {
            std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
            let mut p = wordsmith_manuscript_model::Project::new();
            p.settings.book.title = path.file_stem().map_or_else(|| "Untitled".into(), |n| n.to_string_lossy().into_owned());
            wordsmith_book_format::source::write_owned(&path, wordsmith_book_model::OWNED_SOURCE, &p).map_err(|e| e.to_string())?;
            Source::default()
        }
    };
    let mut b = OpenBook::create(&path, source, &date)?;
    b.edit(|f| {
        f.project.elements = wordsmith_book_model::matter::default_elements();
        Ok(())
    })?;
    b.save()?;
    app.message = b.notice.clone();
    app.book = Some(b);
    app.outline = Default::default();
    Ok(json!({ "path": path.display().to_string() }))
}

/// Run command `id` with `params`.
pub fn run(app: &mut BookApp, id: &str, params: &Value) -> Result<Value, String> {
    match id {
        ids::SAVE => app.open_book()?.save().map(|()| Value::Null),
        ids::UNDO => Ok(json!({ "undone": app.open_book()?.undo() })),
        ids::REDO => Ok(json!({ "redone": app.open_book()?.redo() })),
        ids::OPEN_PATH => app.open_path(s(params, "path")?).map(|()| Value::Null),
        ids::NEW_FROM_TEMPLATE if params.get("template").and_then(Value::as_str) == Some("import") && params.get("file").is_none() => {
            app.message = Some("Choose Import a File… and give the file's path.".into());
            Ok(Value::Null)
        }
        ids::NEW_FROM_TEMPLATE | NEW => new_book(app, params),
        CLOSE => {
            app.book = None;
            Ok(Value::Null)
        }
        FIX_ADD => {
            let section: SectionId = s(params, "section")?.parse().map_err(|e: wordsmith_manuscript_model::BadId| e.to_string())?;
            let note = params.get("note").and_then(Value::as_str).unwrap_or("");
            let (find, replace) = (s(params, "find")?.to_owned(), s(params, "replace")?.to_owned());
            let id = app.open_book()?.add_fix(section, &find, &replace, note, &crate::today())?;
            Ok(json!({ "id": id }))
        }
        FIX_EDIT => {
            let section: SectionId = s(params, "section")?.parse().map_err(|e: wordsmith_manuscript_model::BadId| e.to_string())?;
            let n = params.get("paragraph").and_then(Value::as_u64).and_then(|n| usize::try_from(n).ok()).ok_or("`paragraph` is needed")?;
            let new = s(params, "text")?.to_owned();
            let note = params.get("note").and_then(Value::as_str).unwrap_or("").to_owned();
            let b = app.open_book()?;
            let src = b.source.as_ref().map_err(Clone::clone)?;
            let sec = wordsmith_book_format::fixes::section(&src.book, section).ok_or("there is no such section")?;
            let text = wordsmith_manuscript_model::Text { blocks: sec.content.clone(), notes: sec.notes.clone() };
            let old = text.inline_runs().get(n).map(|r| wordsmith_manuscript_model::text::plain(r)).ok_or("there is no such paragraph")?;
            let (find, replace) = wordsmith_book_format::fixes::from_edit(&old, &new, &text)
                .ok_or("Nothing changed, or the change cannot be told apart from the same words elsewhere.")?;
            let id = b.add_fix(section, &find, &replace, &note, &crate::today())?;
            Ok(json!({ "id": id, "find": find, "replace": replace }))
        }
        FIX_LIST => {
            let b = app.open_book()?;
            let src = b.source.as_ref().map_err(Clone::clone)?;
            let states = wordsmith_book_format::fixes::states(&b.files.project.fixes, src);
            Ok(Value::Array(
                b.files
                    .project
                    .fixes
                    .iter()
                    .zip(states)
                    .map(|(f, st)| json!({ "id": f.id, "find": f.find, "replace": f.replace, "state": st.describe() }))
                    .collect(),
            ))
        }
        PREVIEW_SHOW | PREVIEW_PAGE => preview_command(app, id, params),
        BLOCK_ONLY => block_only(app.open_book()?, params),
        EXPORT => {
            let validator = app.prefs.validator_settings();
            let b = app.open_book()?;
            let editions: Vec<String> = match params.get("editions").and_then(Value::as_array) {
                Some(v) => v.iter().filter_map(Value::as_str).map(str::to_owned).collect(),
                None => b.files.project.editions.iter().map(|e| e.id.clone()).collect(),
            };
            let bundle = params.get("bundle").and_then(Value::as_bool).unwrap_or(false);
            let folder = params.get("folder").and_then(Value::as_str).map_or_else(|| crate::export::default_folder(b), PathBuf::from);
            let paths = crate::export::run(b, &editions, bundle, &folder, Some(&validator))?;
            Ok(json!({ "files": paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>() }))
        }
        PREFLIGHT => {
            let validator = app.prefs.validator_settings();
            let b = app.open_book()?;
            let edition = match params.get("edition").and_then(Value::as_str) {
                Some(e) => e.to_owned(),
                None => b.files.project.editions.first().map(|e| e.id.clone()).ok_or("the book has no editions")?,
            };
            let r = crate::preflight::run(b, &edition, &validator)?;
            Ok(
                json!({ "edition": r.edition, "blocked": r.blocked(), "findings": r.findings.iter().map(crate::preflight::finding_json).collect::<Vec<_>>() }),
            )
        }
        EBOOK_OPEN => ebook_open(app.open_book()?, params.get("open").and_then(Value::as_bool).unwrap_or(true)),
        DESIGNS => Ok(crate::designs::list_json(app.open_book()?)),
        DESIGN_CHOOSE => crate::designs::choose(app.open_book()?, s(params, "id")?).map(|()| Value::Null),
        DESIGN_MIX => {
            crate::designs::mix_part(app.open_book()?, s(params, "part")?, params.get("from").and_then(Value::as_str)).map(|()| Value::Null)
        }
        DESIGN_SAVE_AS => crate::designs::save_as(app.open_book()?, s(params, "name")?).map(|id| json!({ "id": id })),
        DESIGN_EXPORT => crate::designs::export(app.open_book()?, &PathBuf::from(s(params, "path")?)).map(|()| Value::Null),
        DESIGN_IMPORT => crate::designs::import(app.open_book()?, &PathBuf::from(s(params, "path")?)).map(|id| json!({ "id": id })),
        METADATA => Ok(crate::metadata::as_json(app.open_book()?)),
        METADATA_SET => crate::metadata::set(app.open_book()?, s(params, "field")?, s(params, "value")?).map(|()| Value::Null),
        CONTRIBUTOR_ADD => crate::metadata::add_contributor(app.open_book()?, s(params, "name")?, s(params, "role")?).map(|()| Value::Null),
        EDITION_ADD => crate::metadata::add_edition(app.open_book()?, s(params, "format")?).map(|id| json!({ "id": id })),
        EDITION_SET => crate::metadata::set_edition(app.open_book()?, params).map(|()| Value::Null),
        EDITION_REMOVE => {
            let id = s(params, "id")?.to_owned();
            app.open_book()?.edit(|f| {
                let before = f.project.editions.len();
                f.project.editions.retain(|e| e.id != id);
                if f.project.editions.len() == before { Err(format!("there is no edition `{id}`")) } else { Ok(()) }
            })?;
            Ok(Value::Null)
        }
        NUMBERING => {
            let style = |k: &str| -> Result<Option<wordsmith_book_model::NumberStyle>, String> {
                params.get(k).map(|v| serde_json::from_value(v.clone()).map_err(|_| format!("`{v}` is not a number style"))).transpose()
            };
            let (chapters, parts) = (style("chapters")?, style("parts")?);
            let (cl, pl) = (
                params.get("chapter_label").and_then(Value::as_str).map(str::to_owned),
                params.get("part_label").and_then(Value::as_str).map(str::to_owned),
            );
            app.open_book()?.edit(|f| {
                let n = &mut f.project.numbering;
                if let Some(c) = chapters {
                    n.chapters = c;
                }
                if let Some(p) = parts {
                    n.parts = p;
                }
                if let Some(l) = cl {
                    n.chapter_label = l;
                }
                if let Some(l) = pl {
                    n.part_label = l;
                }
                Ok(())
            })?;
            Ok(Value::Null)
        }
        SECTION => {
            let id: SectionId = s(params, "id")?.parse().map_err(|e: wordsmith_manuscript_model::BadId| e.to_string())?;
            let only: Option<wordsmith_book_model::Only> = params
                .get("only")
                .map(|v| serde_json::from_value(v.clone()).map_err(|_| format!("`{v}` is not both, print or ebook")))
                .transpose()?;
            let text = |k: &str| params.get(k).and_then(Value::as_str).map(str::to_owned);
            let (sub, epi, src) = (text("subtitle"), text("epigraph"), text("epigraph_source"));
            let unnumbered = params.get("unnumbered").and_then(Value::as_bool);
            app.open_book()?.edit(|f| {
                let s = f.project.sections.entry(id).or_default();
                if let Some(v) = sub {
                    s.subtitle = v;
                }
                if let Some(v) = epi {
                    s.epigraph = v;
                }
                if let Some(v) = src {
                    s.epigraph_source = v;
                }
                if let Some(v) = unnumbered {
                    s.unnumbered = v;
                }
                if let Some(v) = only {
                    s.only = v;
                }
                if *s == wordsmith_book_model::SectionSettings::default() {
                    f.project.sections.remove(&id);
                }
                Ok(())
            })?;
            Ok(Value::Null)
        }
        CLEANUP_LIST => crate::cleanup::list_json(app.open_book()?),
        CLEANUP_APPLY => {
            let ids: Option<Vec<usize>> = params
                .get("ids")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_u64).filter_map(|n| usize::try_from(n).ok()).collect());
            let n = crate::cleanup::apply(app.open_book()?, ids.as_deref())?;
            app.cleanup = crate::cleanup::Panel::default();
            Ok(json!({ "made": n }))
        }
        CLEANUP_UNDO => Ok(json!({ "undone": app.open_book()?.undo_owned()? })),
        CLEANUP_KEEP => {
            let on = params.get("on").and_then(Value::as_bool).unwrap_or(true);
            app.open_book()?.edit(|f| {
                f.project.cleanup.keep_exactly = on;
                Ok(())
            })?;
            Ok(Value::Null)
        }
        IMPORT_REVIEW => {
            app.review = Some(crate::import_review::Review::open(&PathBuf::from(s(params, "file")?))?);
            Ok(Value::Null)
        }
        IMPORT_CONFIRM => {
            let review = app.review.as_ref().ok_or("nothing is being reviewed")?;
            let (book, report) = review.import()?;
            let out = crate::import_review::report_json(&report);
            app.book = Some(book);
            app.review = None;
            app.last_report = Some(report);
            app.outline = Default::default();
            Ok(out)
        }
        IMPORT => {
            let mut review = crate::import_review::Review::open(&PathBuf::from(s(params, "file")?))?;
            if let Some(path) = params.get("path").and_then(Value::as_str) {
                review.target = path.to_owned();
            }
            for name in params.get("off").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str) {
                review.set_rule(name, false);
            }
            app.review = Some(review);
            run(app, IMPORT_CONFIRM, &Value::Null)
        }
        SAVE_AS_ONE_FILE => {
            let path = PathBuf::from(s(params, "path")?);
            app.open_book()?.save_as_one_file(&path).map(|()| Value::Null)
        }
        BACKUP => app.open_book()?.backup(std::time::SystemTime::now()).map(|p| json!({ "path": p.display().to_string() })),
        RELOAD_SOURCE => Ok(json!({ "changed": app.open_book()?.refresh_source() })),
        OUTLINE => {
            let b = app.open_book()?;
            let rows = outline::rows(&b.files.project, b.source.as_ref().ok().map(|s| &s.book));
            Ok(Value::Array(
                rows.iter().map(|r| json!({ "key": key_json(&r.key), "depth": r.depth, "label": r.label, "movable": r.draggable })).collect(),
            ))
        }
        ADD_ELEMENT => {
            let kind = kind_from(s(params, "kind")?)?;
            let placement = match params.get("placement").and_then(Value::as_str) {
                Some("back") => Placement::Back,
                Some("front") => Placement::Front,
                _ => kind.placement(),
            };
            let b = app.open_book()?;
            let key = serde_json::to_value(kind).ok().and_then(|v| v.as_str().map(str::to_owned)).unwrap_or_else(|| "element".into());
            let mut n = 1;
            let mut new_id = key.clone();
            while b.files.project.elements.iter().any(|e| e.id == new_id) {
                n += 1;
                new_id = format!("{key}-{n}");
            }
            let e = Element { id: new_id.clone(), kind: Some(kind), placement, ..Element::default() };
            b.edit(|f| {
                // Where its kind goes by convention: after the last element on its side that comes
                // before it, else first on its side.
                let rank = kind.rank(placement);
                let els = &f.project.elements;
                let after = els.iter().rposition(|x| x.placement == placement && x.kind.is_some_and(|k| k.rank(placement) <= rank));
                let at = match after {
                    Some(i) => i + 1,
                    None => els.iter().position(|x| x.placement == placement).unwrap_or(els.len()),
                };
                f.project.elements.insert(at, e);
                Ok(())
            })?;
            Ok(json!({ "id": new_id }))
        }
        REMOVE_ELEMENT => {
            let id = s(params, "id")?.to_owned();
            app.open_book()?.edit(|f| {
                let before = f.project.elements.len();
                f.project.elements.retain(|e| e.id != id);
                if f.project.elements.len() == before { Err(format!("there is no element `{id}`")) } else { Ok(()) }
            })?;
            Ok(Value::Null)
        }
        MOVE_ELEMENT => {
            let id = s(params, "id")?.to_owned();
            let target = key_from(params.get("target").unwrap_or(&Value::Null))?;
            let zone = zone_from(params.get("zone").and_then(Value::as_str).unwrap_or("before"))?;
            app.open_book()?.edit(|f| outline::move_element(&mut f.project, &id, &target, zone))?;
            Ok(Value::Null)
        }
        SET_ELEMENT => {
            let id = s(params, "id")?.to_owned();
            let title = params.get("title").and_then(Value::as_str).map(str::to_owned);
            let text = params.get("text").and_then(Value::as_str).map(str::to_owned);
            let only: Option<wordsmith_book_model::Only> = params
                .get("only")
                .map(|v| serde_json::from_value(v.clone()).map_err(|_| format!("`{v}` is not both, print or ebook")))
                .transpose()?;
            let stores: Option<Vec<String>> = params
                .get("stores")
                .map(|v| {
                    v.as_array().map(|a| a.iter().filter_map(Value::as_str).map(str::to_owned).collect()).ok_or("`stores` is a list of store names")
                })
                .transpose()?;
            app.open_book()?.edit(|f| {
                let e = f.project.elements.iter_mut().find(|e| e.id == id).ok_or_else(|| format!("there is no element `{id}`"))?;
                if let Some(t) = title {
                    e.title = t;
                }
                if let Some(t) = text {
                    e.text = t;
                }
                if let Some(o) = only {
                    e.only = o;
                }
                if let Some(v) = stores {
                    e.stores = v;
                }
                Ok(())
            })?;
            Ok(Value::Null)
        }
        other => Err(format!("unknown command {other}")),
    }
}

/// The print preview's commands: build if needed (waiting up to a minute), then answer.
fn preview_command(app: &mut BookApp, id: &str, params: &Value) -> Result<Value, String> {
    let b = app.book.as_ref().ok_or("no book is open")?;
    app.print_preview.update(b);
    app.print_preview.wait(std::time::Duration::from_secs(60));
    app.print_preview.update(b);
    app.print_preview.wait(std::time::Duration::from_secs(60));
    if let Some(e) = &app.print_preview.error {
        return Err(format!("the preview could not be built: {e}"));
    }
    let built = app.print_preview.built.clone().ok_or("the preview is not ready")?;
    if id == PREVIEW_SHOW {
        let section = params.get("section").and_then(Value::as_str).ok_or("give the section's id")?;
        let page = app.print_preview.page_of(section).ok_or_else(|| format!("“{section}” is not in the print edition"))?;
        app.print_preview.scroll_to = Some(page);
        app.outline.selected = Some(outline::key_of_piece(section));
        return Ok(json!({ "page": page + 1 }));
    }
    let n = params.get("page").and_then(Value::as_u64).and_then(|n| usize::try_from(n).ok()).ok_or("give the page number")?;
    let piece = (n >= 1 && n <= built.print.typeset.page_count())
        .then(|| built.print.typeset.section_of_page(n - 1))
        .flatten()
        .ok_or_else(|| format!("there is no page {n}"))?;
    Ok(json!({ "section": piece }))
}

/// Build the book's first ebook edition into the system's temporary folder and, with `open`,
/// hand it to the reading app the user has installed.
fn ebook_open(b: &mut OpenBook, open: bool) -> Result<Value, String> {
    let src = b.source.as_ref().map_err(Clone::clone)?;
    let project = &b.files.project;
    let edition = project.editions.iter().find(|e| !e.format.is_print()).cloned().unwrap_or_else(wordsmith_book_model::Edition::ebook);
    let book = wordsmith_book_format::fixes::book_for_build(&project.fixes, src);
    let (assembled, ebook) = wordsmith_book_check::build::ebook(project, &b.files.designs, &book, &edition, &|p| b.picture(p))?;
    let name: String = assembled.meta.title.chars().filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-').collect();
    let name = if name.trim().is_empty() { "Book".to_owned() } else { name.trim().to_owned() };
    let dir = std::env::temp_dir().join("wordsmith-book-preview");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{name}.epub"));
    std::fs::write(&path, &ebook.bytes).map_err(|e| e.to_string())?;
    if open {
        open_with_system(&path)?;
    }
    Ok(json!({ "path": path.display().to_string() }))
}

/// Hand a file to the app the system opens it with.
fn open_with_system(path: &std::path::Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut cmd = std::process::Command::new("open");
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", "start", ""]);
        c
    };
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let mut cmd = std::process::Command::new("xdg-open");
    cmd.arg(path).spawn().map(|_| ()).map_err(|e| format!("no reading app could be started: {e}"))
}

/// Set or clear a paragraph's only-in rule (STR-9).
fn block_only(b: &mut OpenBook, params: &Value) -> Result<Value, String> {
    let section: SectionId = s(params, "section")?.parse().map_err(|e: wordsmith_manuscript_model::BadId| e.to_string())?;
    let starts = s(params, "starts")?.to_owned();
    let only: wordsmith_book_model::Only =
        serde_json::from_value(params.get("only").cloned().unwrap_or(Value::Null)).map_err(|_| "`only` is print, ebook or both".to_owned())?;
    if starts.trim().is_empty() {
        return Err("give the words the paragraph starts with".into());
    }
    let src = b.source.as_ref().map_err(Clone::clone)?;
    let sec = wordsmith_book_format::fixes::section(&src.book, section).ok_or("there is no such section")?;
    fn texts(s: &wordsmith_manuscript_format::book::Section, out: &mut Vec<String>) {
        out.extend(s.content.iter().map(wordsmith_book_format::assemble::block_text));
        for c in &s.children {
            texts(c, out);
        }
    }
    let mut all = Vec::new();
    texts(sec, &mut all);
    let n = all.iter().filter(|t| t.trim_start().starts_with(starts.as_str())).count();
    if n != 1 {
        return Err(format!("“{starts}” starts {n} paragraphs of that section; give words that start exactly one"));
    }
    b.edit(|f| {
        let mut settings = f.project.sections.get(&section).cloned().unwrap_or_default();
        settings.blocks_only.retain(|r| r.starts != starts);
        if only != wordsmith_book_model::Only::Both {
            settings.blocks_only.push(wordsmith_book_model::BlockOnly { starts: starts.clone(), only });
        }
        f.project.sections.insert(section, settings);
        Ok(())
    })?;
    Ok(Value::Null)
}
