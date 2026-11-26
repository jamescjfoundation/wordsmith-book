// SPDX-License-Identifier: MIT OR Apache-2.0
//! The ebook preview (EBK-8, plan 5.8): the first chapter in the design's ebook mapping at a
//! phone, e-ink reader or tablet size, with text size and dark-page toggles, page by page, and
//! "Open in a Reading App" for the exported EPUB. It is labelled a preview: reading apps lay
//! ebooks out their own way. Layout and drawing run on a worker thread.

use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use jcj_shell::egui;
use jcj_shell::i18n::{tr, tr_with};
use wordsmith_book_typeset::screen::{self, DEVICES, TEXT_SIZES};

use crate::doc::OpenBook;

/// One laid-out chapter, drawn.
pub struct Laid {
    /// What it was laid out for: (revision, device, text size index).
    pub key: (u64, &'static str, usize),
    /// Each page: width, height and RGBA pixels (light page).
    pub pages: Vec<(u32, u32, Vec<u8>)>,
}

/// The preview's settings and its latest layout.
pub struct EbookPreview {
    /// The device shown (an id of [`DEVICES`]).
    pub device: &'static str,
    /// The text size (an index into [`TEXT_SIZES`]).
    pub text: usize,
    /// Dark page: light text on a dark page.
    pub dark: bool,
    /// The page shown (from 0).
    pub page: usize,
    /// The latest layout.
    pub laid: Option<Laid>,
    /// Why the last layout failed.
    pub error: Option<String>,
    job: Option<((u64, &'static str, usize), Receiver<Result<Laid, String>>)>,
    texture: Option<((u64, &'static str, usize, usize, bool), egui::TextureHandle)>,
}

impl Default for EbookPreview {
    fn default() -> Self {
        EbookPreview { device: "phone", text: 1, dark: false, page: 0, laid: None, error: None, job: None, texture: None }
    }
}

/// Pixels for a dark page: light text on a near-black page.
pub fn darken(pixels: &[u8]) -> Vec<u8> {
    let mut out = pixels.to_vec();
    for px in out.as_chunks_mut::<4>().0 {
        for c in &mut px[..3] {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "0 to 255")]
            let v = (24.0 + f32::from(255 - *c) * 0.82).round() as u8;
            *c = v;
        }
    }
    out
}

/// The mean brightness of RGBA pixels, 0 to 255.
pub fn brightness(pixels: &[u8]) -> f64 {
    let n = pixels.len() / 4;
    if n == 0 {
        return 0.0;
    }
    let sum: u64 = pixels.as_chunks::<4>().0.iter().map(|p| u64::from(p[0]) + u64::from(p[1]) + u64::from(p[2])).sum();
    #[allow(clippy::cast_precision_loss, reason = "an average")]
    let mean = sum as f64 / (3 * n) as f64;
    mean
}

impl EbookPreview {
    fn key(&self, b: &OpenBook) -> (u64, &'static str, usize) {
        (b.revision, self.device, self.text)
    }

    /// Whether a layout is running.
    pub fn busy(&self) -> bool {
        self.job.is_some()
    }

    /// Start a layout when the book or a setting changed.
    pub fn update(&mut self, b: &OpenBook) {
        self.poll();
        let key = self.key(b);
        if self.laid.as_ref().is_some_and(|l| l.key == key) || self.job.as_ref().is_some_and(|(k, _)| *k == key) {
            return;
        }
        if self.job.is_some() {
            return;
        }
        let Ok(src) = &b.source else { return };
        let project = b.files.project.clone();
        let designs = b.files.designs.clone();
        let book = wordsmith_book_format::fixes::book_for_build(&project.fixes, src);
        let pictures: std::collections::BTreeMap<String, Vec<u8>> =
            crate::preview::pictures_of(&book).into_iter().filter_map(|p| b.picture(&p).map(|x| (p, x))).collect();
        let lang = if book.metadata.info.language.is_empty() { "en".to_owned() } else { book.metadata.info.language.clone() };
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let out = (|| {
                let device = screen::device(key.1).ok_or("no such device")?;
                let design = wordsmith_book_design::mix::resolve(&project.design, &designs).map_err(|e| e.to_string())?;
                let ebook = project.editions.iter().find(|e| !e.format.is_print()).cloned();
                let a = wordsmith_book_format::assemble::assemble(&project, &book, ebook.as_ref());
                let t = screen::first_chapter(&a, &design, &device, TEXT_SIZES[key.2].1, &lang, &|p| pictures.get(p).cloned())
                    .map_err(|e| e.to_string())?;
                let mut pages = Vec::new();
                for i in 0..t.page_count().min(400) {
                    let img = t.page_image(i, 110.0).map_err(|e| e.to_string())?;
                    pages.push((img.width, img.height, img.pixels));
                }
                Ok(Laid { key, pages })
            })();
            let _ = tx.send(out);
        });
        self.job = Some((key, rx));
    }

    /// Take a finished layout.
    pub fn poll(&mut self) {
        let Some((_, rx)) = &self.job else { return };
        match rx.try_recv() {
            Ok(Ok(l)) => {
                self.page = self.page.min(l.pages.len().saturating_sub(1));
                self.laid = Some(l);
                self.error = None;
                self.job = None;
            }
            Ok(Err(e)) => {
                self.error = Some(e);
                self.job = None;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.error = Some("the layout stopped".into());
                self.job = None;
            }
        }
    }

    /// Wait for the running layout (tests and agent tools).
    pub fn wait(&mut self, limit: Duration) -> bool {
        let start = std::time::Instant::now();
        while self.job.is_some() && start.elapsed() < limit {
            self.poll();
            std::thread::sleep(Duration::from_millis(5));
        }
        self.job.is_none()
    }

    /// The pixels of the page shown, as drawn (dark when dark page is on).
    pub fn shown_pixels(&self) -> Option<(u32, u32, Vec<u8>)> {
        let (w, h, px) = self.laid.as_ref()?.pages.get(self.page)?;
        Some((*w, *h, if self.dark { darken(px) } else { px.clone() }))
    }

    /// Draw the preview; gives a command to run.
    pub fn ui(&mut self, ui: &mut egui::Ui, b: &OpenBook) -> Option<(&'static str, serde_json::Value)> {
        self.update(b);
        let mut run = None;
        ui.label(egui::RichText::new(tr("Preview: reading apps lay out ebooks their own way.")).italics());
        ui.horizontal_wrapped(|ui| {
            for d in DEVICES {
                if ui.selectable_label(self.device == d.id, d.name).clicked() {
                    self.device = d.id;
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(tr("Text size:"));
            for (i, (name, _)) in TEXT_SIZES.iter().enumerate() {
                if ui.selectable_label(self.text == i, *name).clicked() {
                    self.text = i;
                }
            }
        });
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.dark, tr("Dark page"));
            if ui.button(tr("Open in a Reading App")).clicked() {
                run = Some((crate::commands::EBOOK_OPEN, serde_json::json!({})));
            }
        });
        if self.busy() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(tr("Laying out the chapter…"));
            });
            ui.ctx().request_repaint_after(Duration::from_millis(50));
        }
        if let Some(e) = &self.error {
            ui.colored_label(ui.visuals().error_fg_color, tr_with("The preview could not be made: {why}", &[("why", &e.to_string())]));
        }
        let pages = self.laid.as_ref().map_or(0, |l| l.pages.len());
        if pages == 0 {
            return run;
        }
        ui.horizontal(|ui| {
            if ui.add_enabled(self.page > 0, egui::Button::new(tr("Previous Page"))).clicked() {
                self.page -= 1;
            }
            let (at, of) = ((self.page + 1).to_string(), pages.to_string());
            ui.label(tr_with("Page {page} of {pages}", &[("page", &at), ("pages", &of)]));
            if ui.add_enabled(self.page + 1 < pages, egui::Button::new(tr("Next Page"))).clicked() {
                self.page += 1;
            }
        });
        let Some(laid) = &self.laid else { return run };
        let tkey = (laid.key.0, laid.key.1, laid.key.2, self.page, self.dark);
        if self.texture.as_ref().is_none_or(|(k, _)| *k != tkey)
            && let Some((w, h, px)) = self.shown_pixels()
        {
            let size = [usize::try_from(w).unwrap_or(0), usize::try_from(h).unwrap_or(0)];
            let tex = ui.ctx().load_texture("ebook-page", egui::ColorImage::from_rgba_unmultiplied(size, &px), egui::TextureOptions::LINEAR);
            self.texture = Some((tkey, tex));
        }
        if let Some((_, tex)) = &self.texture {
            let [w, h] = tex.size();
            #[allow(clippy::cast_precision_loss, reason = "image sizes")]
            let (w, h) = (w as f32, h as f32);
            let scale = (ui.available_width() / w).min(1.0);
            let frame = if self.dark { egui::Color32::from_rgb(24, 24, 24) } else { egui::Color32::WHITE };
            egui::Frame::new()
                .fill(frame)
                .stroke(egui::Stroke::new(6.0, egui::Color32::from_rgb(50, 50, 55)))
                .corner_radius(10.0)
                .inner_margin(4.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::Image::new((tex.id(), egui::vec2(w * scale, h * scale))).alt_text(tr("The chapter's page as the preview lays it out")),
                    );
                });
        }
        run
    }
}
