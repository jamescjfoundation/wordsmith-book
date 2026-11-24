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
/// Change an element's title or text: `{"id", "title"?, "text"?}`.
pub const SET_ELEMENT: &str = "book.element.set";

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
                f.project.elements.push(e);
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
            app.open_book()?.edit(|f| {
                let e = f.project.elements.iter_mut().find(|e| e.id == id).ok_or_else(|| format!("there is no element `{id}`"))?;
                if let Some(t) = title {
                    e.title = t;
                }
                if let Some(t) = text {
                    e.text = t;
                }
                Ok(())
            })?;
            Ok(Value::Null)
        }
        other => Err(format!("unknown command {other}")),
    }
}
