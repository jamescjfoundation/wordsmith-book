// SPDX-License-Identifier: MIT OR Apache-2.0
//! The design chooser (DSN-5): every design the book can use, filtered by genre and mood, each
//! with a preview of the book's own first chapter. Choosing one is one undo step.

use jcj_shell::egui;
use jcj_shell::i18n::{tr, tr_with};
use serde_json::{Value, json};
use wordsmith_book_design::Design;

use crate::doc::OpenBook;

/// The chooser's filters.
#[derive(Default)]
pub struct Chooser {
    /// Show only designs for this genre.
    pub genre: Option<String>,
    /// Show only designs of this mood.
    pub mood: Option<String>,
}

/// The designs that pass the filters.
pub fn filter<'a>(designs: &'a [Design], genre: Option<&str>, mood: Option<&str>) -> Vec<&'a Design> {
    designs
        .iter()
        .filter(|d| genre.is_none_or(|g| d.genres.iter().any(|x| x == g)))
        .filter(|d| mood.is_none_or(|m| d.moods.iter().any(|x| x == m)))
        .collect()
}

/// Every genre and every mood the designs name, sorted.
pub fn tags(designs: &[Design]) -> (Vec<String>, Vec<String>) {
    let mut g: Vec<String> = designs.iter().flat_map(|d| d.genres.clone()).collect();
    let mut m: Vec<String> = designs.iter().flat_map(|d| d.moods.clone()).collect();
    g.sort();
    g.dedup();
    m.sort();
    m.dedup();
    (g, m)
}

/// The opening of the book's first chapter: its title and first paragraph.
pub fn first_chapter(b: &OpenBook) -> (String, String) {
    let Ok(src) = &b.source else { return (String::new(), String::new()) };
    let a = wordsmith_book_format::assemble::assemble(&b.files.project, &src.book, None);
    let Some(ch) = a.body.iter().find(|p| p.what == wordsmith_book_format::assemble::What::Chapter) else { return (String::new(), String::new()) };
    let title = [ch.label.clone().unwrap_or_default(), ch.title.clone()].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" · ");
    let text = wordsmith_manuscript_model::Text { blocks: ch.blocks.clone(), notes: vec![] }
        .plain_paragraphs()
        .into_iter()
        .find(|p| !p.trim().is_empty())
        .unwrap_or_default();
    (title, text.chars().take(320).collect())
}

fn preview(ui: &mut egui::Ui, d: &Design, title: &str, text: &str) {
    let scale = |pt: f32| (pt * 0.9).clamp(8.0, 26.0);
    let align = if d.chapter.align == wordsmith_book_design::design::Align::Center { egui::Align::Center } else { egui::Align::Min };
    egui::Frame::new().fill(egui::Color32::from_rgb(252, 250, 244)).inner_margin(12.0).corner_radius(4.0).show(ui, |ui| {
        ui.set_width(260.0);
        ui.with_layout(egui::Layout::top_down(align), |ui| {
            let t = match d.chapter.title_case {
                wordsmith_book_design::design::Case::Capitals => title.to_uppercase(),
                _ => title.to_owned(),
            };
            ui.label(egui::RichText::new(t).size(scale(d.chapter.title_size.min(24.0))).color(egui::Color32::from_rgb(30, 30, 30)));
            if !d.chapter.ornament.is_empty() {
                ui.label(egui::RichText::new(&d.chapter.ornament).size(14.0).color(egui::Color32::from_rgb(30, 30, 30)));
            }
        });
        ui.add_space(6.0);
        ui.label(egui::RichText::new(text).size(scale(d.body.size)).color(egui::Color32::from_rgb(30, 30, 30)));
    });
}

impl Chooser {
    /// Draw the chooser; gives a command to run.
    pub fn ui(&mut self, ui: &mut egui::Ui, b: &OpenBook) -> Option<(&'static str, Value)> {
        let designs = crate::designs::available(b);
        let (genres, moods) = tags(&designs);
        let mut run = None;
        ui.horizontal_wrapped(|ui| {
            ui.label(tr("Genre:"));
            if ui.selectable_label(self.genre.is_none(), tr("Any")).clicked() {
                self.genre = None;
            }
            for g in &genres {
                if ui.selectable_label(self.genre.as_ref() == Some(g), g.replace('-', " ")).clicked() {
                    self.genre = Some(g.clone());
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(tr("Mood:"));
            if ui.selectable_label(self.mood.is_none(), tr("Any mood")).clicked() {
                self.mood = None;
            }
            for m in &moods {
                if ui.selectable_label(self.mood.as_ref() == Some(m), m).clicked() {
                    self.mood = Some(m.clone());
                }
            }
        });
        ui.separator();
        let (title, text) = first_chapter(b);
        let chosen = b.files.project.design.id.clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for d in filter(&designs, self.genre.as_deref(), self.mood.as_deref()) {
                ui.horizontal(|ui| {
                    ui.strong(&d.name);
                    if d.id == chosen {
                        ui.label(tr("(in use)"));
                    } else if ui.button(tr_with("Choose {design}", &[("design", &d.name)])).clicked() {
                        run = Some((crate::commands::DESIGN_CHOOSE, json!({ "id": d.id })));
                    }
                });
                ui.label(&d.about);
                preview(ui, d, &title, &text);
                ui.add_space(10.0);
            }
        });
        run
    }
}
