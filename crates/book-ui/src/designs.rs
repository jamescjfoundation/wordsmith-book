// SPDX-License-Identifier: MIT OR Apache-2.0
//! Designs in the app: choosing one (DSN-5), taking a part from another (DSN-3), saving the
//! result as the user's own and exporting or importing a design as one file (DSN-4).

use std::path::Path;

use serde_json::{Value, json};
use wordsmith_book_design::design::{self, Design};
use wordsmith_book_design::{builtin, known_role, mix};
use wordsmith_book_model::DesignPart;

use crate::doc::OpenBook;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// Every design the book can use: the built-in ones and the project's own.
pub fn available(b: &OpenBook) -> Vec<Design> {
    let mut out = builtin::all();
    out.extend(b.files.designs.values().filter_map(|t| design::read(t, &known_role).ok()));
    out
}

/// The design the book uses now.
pub fn current(b: &OpenBook) -> Result<Design, String> {
    mix::resolve(&b.files.project.design, &b.files.designs).map_err(err)
}

/// The designs as JSON (for agents and the chooser).
pub fn list_json(b: &OpenBook) -> Value {
    let chosen = &b.files.project.design.id;
    Value::Array(
        available(b)
            .iter()
            .map(|d| json!({ "id": d.id, "name": d.name, "about": d.about, "genres": d.genres, "moods": d.moods, "chosen": d.id == *chosen, "own": b.files.designs.contains_key(&d.id) }))
            .collect(),
    )
}

/// Choose a design (one undo step); its mixed-in parts are cleared.
pub fn choose(b: &mut OpenBook, id: &str) -> Result<(), String> {
    mix::find(id, &b.files.designs).map_err(err)?;
    let id = id.to_owned();
    b.edit(|f| {
        f.project.design.id = id;
        f.project.design.mix.clear();
        Ok(())
    })
}

/// Take one part of the book's design from another design (or put it back with `from` = none).
pub fn mix_part(b: &mut OpenBook, part: &str, from: Option<&str>) -> Result<(), String> {
    let part: DesignPart = serde_json::from_value(Value::String(part.to_owned())).map_err(|_| format!("`{part}` is not a part of a design (body, chapter-opening, scene-breaks, running-heads)"))?;
    if let Some(id) = from {
        mix::find(id, &b.files.designs).map_err(err)?;
    }
    let from = from.map(str::to_owned);
    b.edit(|f| {
        match from {
            Some(id) => f.project.design.mix.insert(part, id),
            None => f.project.design.mix.remove(&part),
        };
        Ok(())
    })
}

/// Save the book's design as the user's own under `name`; the book then uses it. Gives its id.
pub fn save_as(b: &mut OpenBook, name: &str) -> Result<String, String> {
    let mut d = current(b)?;
    d.id = mix::free_id(name, &b.files.designs);
    d.name = name.trim().to_owned();
    let problems = design::problems(&d, &known_role);
    if let Some(p) = problems.first() {
        return Err(p.clone());
    }
    let text = design::write(&d).map_err(err)?;
    let id = d.id.clone();
    let id2 = id.clone();
    b.edit(|f| {
        f.designs.insert(id2.clone(), text);
        f.project.design.id = id2;
        f.project.design.mix.clear();
        Ok(())
    })?;
    Ok(id)
}

/// Write the book's design to `path` as one design file.
pub fn export(b: &OpenBook, path: &Path) -> Result<(), String> {
    let text = design::write(&current(b)?).map_err(err)?;
    write_file(path, text.as_bytes())
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    wordsmith_book_format::pack::write_file(path, bytes).map_err(err)
}

/// Import a design file into the project (under a free id); gives its id.
pub fn import(b: &mut OpenBook, path: &Path) -> Result<String, String> {
    let meta = std::fs::metadata(path).map_err(err)?;
    if meta.len() > design::MAX_FILE as u64 {
        return Err("The file is too large to be a design.".into());
    }
    let text = std::fs::read_to_string(path).map_err(err)?;
    let mut d = design::read(&text, &known_role).map_err(err)?;
    d.id = mix::free_id(&d.id, &b.files.designs);
    let text = design::write(&d).map_err(err)?;
    let id = d.id.clone();
    let id2 = id.clone();
    b.edit(|f| {
        f.designs.insert(id2, text);
        Ok(())
    })?;
    Ok(id)
}
