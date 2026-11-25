// SPDX-License-Identifier: MIT OR Apache-2.0
//! Metadata and editions (META-1, META-2): the source's metadata with Book's additions, the
//! editions with their ISBNs (checked), trims, paper and price notes, and what the book still
//! needs. Book's additions live in `book.toml`; a linked source is never written.

use jcj_shell::egui;
use serde_json::{Value, json};
use wordsmith_book_format::needs::needs;
use wordsmith_book_model::{Contributor, Edition, Format, Imprint, Paper, Role, isbn};

use crate::doc::OpenBook;

/// Fields that replace the source's metadata in Book's builds (the source keeps its own).
pub const OVERRIDES: [&str; 6] = ["title", "subtitle", "byline", "rights", "description", "about"];

/// The metadata and needs as JSON.
pub fn as_json(b: &OpenBook) -> Value {
    let p = &b.files.project;
    let source = b.source.as_ref().ok().map(|s| {
        let i = &s.book.metadata.info;
        json!({ "title": i.title, "subtitle": i.subtitle, "byline": s.book.metadata.byline, "language": i.language, "series": i.series,
                "publisher": i.publisher, "copyright_year": i.copyright_year, "rights": i.rights, "description": i.description })
    });
    let needs: Vec<Value> = b
        .source
        .as_ref()
        .ok()
        .map(|s| {
            needs(p, &s.book).into_iter().map(|n| json!({ "severity": n.severity.title(), "message": n.message, "edition": n.edition })).collect()
        })
        .unwrap_or_default();
    json!({ "source": source, "additions": p.metadata, "editions": p.editions, "needs": needs })
}

/// Set one field of Book's metadata additions.
pub fn set(b: &mut OpenBook, field: &str, value: &str) -> Result<(), String> {
    let field = field.to_owned();
    let value = value.to_owned();
    b.edit(|f| {
        let m = &mut f.project.metadata;
        match field.as_str() {
            k if OVERRIDES.contains(&k) => {
                if value.is_empty() {
                    m.overrides.remove(k);
                } else {
                    m.overrides.insert(k.to_owned(), value);
                }
            }
            "copyright_holder" => m.copyright_holder = value,
            "about_author" => m.about_author = value,
            "subject_codes" => m.subject_codes = value.split(',').map(|s| s.trim().to_owned()).filter(|s| !s.is_empty()).collect(),
            "imprint" | "imprint_address" | "imprint_website" => {
                let imp = m.imprint.get_or_insert_with(Imprint::default);
                match field.as_str() {
                    "imprint" => imp.name = value,
                    "imprint_address" => imp.address = value.split('\n').map(str::to_owned).filter(|s| !s.is_empty()).collect(),
                    _ => imp.website = value,
                }
                if *imp == Imprint::default() {
                    m.imprint = None;
                }
            }
            other => return Err(format!("`{other}` is not a metadata field Book keeps")),
        }
        Ok(())
    })
}

/// Add a contributor with a role.
pub fn add_contributor(b: &mut OpenBook, name: &str, role: &str) -> Result<(), String> {
    let role: Role = serde_json::from_value(Value::String(role.to_owned())).map_err(|_| format!("`{role}` is not a contributor's role"))?;
    let name = name.trim().to_owned();
    if name.is_empty() {
        return Err("A contributor needs a name.".into());
    }
    b.edit(|f| {
        f.project.metadata.contributors.push(Contributor { name, role });
        Ok(())
    })
}

/// Add an edition of `format`; gives its id.
pub fn add_edition(b: &mut OpenBook, format: &str) -> Result<String, String> {
    let format: Format = serde_json::from_value(Value::String(format.to_owned())).map_err(|_| format!("`{format}` is not an edition's format"))?;
    let base = serde_json::to_value(format).ok().and_then(|v| v.as_str().map(str::to_owned)).unwrap_or_else(|| "edition".into());
    let mut id = base.clone();
    let mut n = 1;
    while b.files.project.editions.iter().any(|e| e.id == id) {
        n += 1;
        id = format!("{base}-{n}");
    }
    let mut e = match format {
        Format::Ebook => Edition::ebook(),
        Format::Hardcover => Edition::hardcover(),
        _ => Edition::paperback(),
    };
    e.id.clone_from(&id);
    e.format = format;
    e.name = if n == 1 { format.title().to_owned() } else { format!("{} {n}", format.title()) };
    b.edit(|f| {
        f.project.editions.push(e);
        Ok(())
    })?;
    Ok(id)
}

/// Change an edition: `{"id", "name", "isbn", "trim", "paper", "profile", "bleed", "notes", "price_notes", "store"}`.
/// An ISBN is checked before it is kept.
pub fn set_edition(b: &mut OpenBook, params: &Value) -> Result<(), String> {
    let id = params.get("id").and_then(Value::as_str).ok_or("`id` is needed")?.to_owned();
    let text = |k: &str| params.get(k).and_then(Value::as_str).map(str::to_owned);
    let isbn_value = text("isbn");
    if let Some(i) = &isbn_value
        && !i.trim().is_empty()
    {
        isbn::check(i)?;
    }
    let paper: Option<Paper> =
        params.get("paper").map(|v| serde_json::from_value(v.clone()).map_err(|_| format!("`{v}` is not white, cream or color"))).transpose()?;
    let (name, trim, profile, price, store) = (text("name"), text("trim"), text("profile"), text("price_notes"), text("store"));
    let bleed = params.get("bleed").and_then(Value::as_bool);
    let notes: Option<wordsmith_book_model::NoteStyle> = params
        .get("notes")
        .map(|v| serde_json::from_value(v.clone()).map_err(|_| format!("`{v}` is not popup, chapter-end or book-end")))
        .transpose()?;
    b.edit(|f| {
        let e = f.project.editions.iter_mut().find(|e| e.id == id).ok_or_else(|| format!("there is no edition `{id}`"))?;
        if let Some(v) = name {
            e.name = v;
        }
        if let Some(v) = isbn_value {
            e.isbn = v;
        }
        if let Some(v) = trim {
            e.trim = v;
        }
        if let Some(v) = profile {
            e.profile = v;
        }
        if let Some(v) = price {
            e.price_notes = v;
        }
        if let Some(v) = store {
            e.store = v;
        }
        if paper.is_some() {
            e.paper = paper;
        }
        if let Some(v) = bleed {
            e.bleed = v;
        }
        if let Some(v) = notes {
            e.notes = v;
        }
        // A printed edition's trim must be one its profile has (an empty one is a need).
        if e.format.is_print() && !e.trim.is_empty() {
            let pid = e.profile_id();
            let prof = wordsmith_book_check::profile::get(pid).ok_or_else(|| format!("there is no print profile “{pid}”"))?;
            if prof.trim(&e.trim).is_none() {
                let ids: Vec<&str> = prof.trims.iter().map(|t| t.id.as_str()).collect();
                return Err(format!("the {} profile has no “{}” trim; it has {}", prof.name.to_lowercase(), e.trim, ids.join(", ")));
            }
        }
        Ok(())
    })
}

/// The metadata panel's edits in progress.
#[derive(Default)]
pub struct Panel {
    isbn: std::collections::BTreeMap<String, String>,
}

impl Panel {
    /// Draw the panel; gives a command to run.
    pub fn ui(&mut self, ui: &mut egui::Ui, b: &OpenBook) -> Option<(&'static str, Value)> {
        let mut run = None;
        if let Ok(s) = &b.source {
            let i = &s.book.metadata.info;
            ui.strong("From the manuscript");
            ui.label(format!("{} — {}", i.title, s.book.metadata.byline));
            if !i.subtitle.is_empty() {
                ui.label(&i.subtitle);
            }
            ui.label(format!("Language: {}", i.language));
            ui.separator();
            ui.strong("Still needed");
            for n in needs(&b.files.project, &s.book) {
                ui.label(format!("{}: {}", n.severity.title(), n.message));
            }
        }
        ui.separator();
        ui.strong("Editions");
        for e in &b.files.project.editions {
            ui.horizontal(|ui| {
                ui.label(format!("{} ({})", e.name, e.format.title()));
                let v = self.isbn.entry(e.id.clone()).or_insert_with(|| e.isbn.clone());
                let label = ui.label("ISBN");
                let r = ui.add(egui::TextEdit::singleline(v).desired_width(150.0)).labelled_by(label.id);
                if r.lost_focus() && *v != e.isbn {
                    run = Some((crate::commands::EDITION_SET, json!({ "id": e.id, "isbn": v.clone() })));
                }
                if !v.trim().is_empty()
                    && let Err(m) = isbn::check(v)
                {
                    ui.label(m);
                }
            });
        }
        ui.horizontal(|ui| {
            if ui.button("Add Paperback").clicked() {
                run = Some((crate::commands::EDITION_ADD, json!({ "format": "paperback" })));
            }
            if ui.button("Add Hardcover").clicked() {
                run = Some((crate::commands::EDITION_ADD, json!({ "format": "hardcover" })));
            }
            if ui.button("Add Ebook").clicked() {
                run = Some((crate::commands::EDITION_ADD, json!({ "format": "ebook" })));
            }
        });
        run
    }
}
