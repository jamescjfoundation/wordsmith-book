// SPDX-License-Identifier: MIT OR Apache-2.0
//! The cleanup panel (IMP-6): the changes proposed for the book's own source, each with its place
//! and shown word by word, chosen one by one or all at once, or none ("Keep exactly as written").

use std::collections::BTreeSet;

use jcj_shell::egui;
use jcj_shell::i18n::{tr, tr_with, trn};
use serde_json::{Value, json};
use wordsmith_book_import::cleanup::{self, Change};

use crate::doc::OpenBook;

/// The panel's state: which changes are chosen (all, until the user says otherwise).
#[derive(Default)]
pub struct Panel {
    /// Changes left out (undone) by the user, by id.
    pub undone: BTreeSet<usize>,
    /// The change shown word by word.
    pub shown: Option<usize>,
}

/// The changes proposed for the book's own source now.
pub fn proposed(b: &OpenBook) -> Result<Vec<Change>, String> {
    Ok(cleanup::propose(b.owned_project()?, &b.files.project.cleanup))
}

fn title_of(b: &OpenBook, c: &Change) -> String {
    b.owned_project().ok().and_then(|p| p.binder.get(c.item)).map_or_else(String::new, |i| i.title.clone())
}

/// The list as JSON.
pub fn list_json(b: &OpenBook) -> Result<Value, String> {
    let changes = proposed(b)?;
    Ok(Value::Array(
        changes.iter().map(|c| json!({ "id": c.id, "kind": c.kind.key(), "section": title_of(b, c), "from": c.from, "to": c.to })).collect(),
    ))
}

/// Make the chosen changes in the book's own source.
pub fn apply(b: &mut OpenBook, ids: Option<&[usize]>) -> Result<usize, String> {
    let changes = proposed(b)?;
    let chosen: BTreeSet<usize> = match ids {
        Some(ids) => ids.iter().copied().collect(),
        None => cleanup::all(&changes),
    };
    let n = changes.iter().filter(|c| chosen.contains(&c.id)).count();
    if n == 0 {
        return Ok(0);
    }
    let new = cleanup::apply(b.owned_project()?, &changes, &chosen);
    b.write_owned(&new)?;
    Ok(n)
}

impl Panel {
    /// Draw the panel; gives the command to run, if any.
    pub fn ui(&mut self, ui: &mut egui::Ui, b: &OpenBook) -> Option<(&'static str, Value)> {
        let mut run = None;
        let mut keep = b.files.project.cleanup.keep_exactly;
        if ui.checkbox(&mut keep, tr("Keep exactly as written")).changed() {
            run = Some((crate::commands::CLEANUP_KEEP, json!({ "on": keep })));
        }
        let changes = match proposed(b) {
            Ok(c) => c,
            Err(e) => {
                ui.label(e);
                return run;
            }
        };
        let chosen = (changes.len() - changes.iter().filter(|c| self.undone.contains(&c.id)).count()).to_string();
        ui.label(trn(
            "{n} change proposed; {chosen} chosen.",
            "{n} changes proposed; {chosen} chosen.",
            changes.len() as u64,
            &[("chosen", &chosen)],
        ));
        ui.horizontal(|ui| {
            if ui.button(tr("Choose All")).clicked() {
                self.undone.clear();
            }
            if ui.button(tr("Choose None")).clicked() {
                self.undone = cleanup::all(&changes);
            }
        });
        ui.horizontal(|ui| {
            if ui.button(tr("Make Chosen Changes")).clicked() {
                let ids: Vec<usize> = changes.iter().filter(|c| !self.undone.contains(&c.id)).map(|c| c.id).collect();
                run = Some((crate::commands::CLEANUP_APPLY, json!({ "ids": ids })));
            }
            if ui.button(tr("Undo Last Cleanup")).clicked() {
                run = Some((crate::commands::CLEANUP_UNDO, json!({})));
            }
        });
        if let Some(id) = self.shown
            && let (Some(c), Ok(p)) = (changes.iter().find(|c| c.id == id), b.owned_project())
        {
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                for s in cleanup::shown(p, c) {
                    let text = egui::RichText::new(s.text.as_str());
                    ui.label(match s.change {
                        jcj_features::wordiff::Change::Same => text,
                        jcj_features::wordiff::Change::Added => text.underline(),
                        jcj_features::wordiff::Change::Removed => text.strikethrough(),
                    });
                }
            });
        }
        ui.separator();
        egui::ScrollArea::vertical().show_rows(ui, 20.0, changes.len(), |ui, range| {
            for c in &changes[range] {
                ui.horizontal(|ui| {
                    let mut on = !self.undone.contains(&c.id);
                    let (from, to) = (c.from.replace(' ', "␣"), c.to.replace(' ', "␣"));
                    let label = tr_with(
                        "{kind}: “{from}” → “{to}” in {section}",
                        &[("kind", &tr(c.kind.title())), ("from", &from), ("to", &to), ("section", &title_of(b, c))],
                    );
                    if ui.checkbox(&mut on, label).changed() {
                        if on {
                            self.undone.remove(&c.id);
                        } else {
                            self.undone.insert(c.id);
                        }
                    }
                    if ui.small_button(tr("Show")).clicked() {
                        self.shown = Some(c.id);
                    }
                });
            }
        });
        run
    }
}
