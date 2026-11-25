// SPDX-License-Identifier: MIT OR Apache-2.0
//! The print preview (PRV-1, SPEC section 7): the edition laid out as spreads (the first page
//! alone on the right, then verso left and recto right), drawn from the core's page images, with
//! trim, margin and bleed guides that can be turned off, and zoom. Clicking a page opens its
//! section; "Show in preview" goes to a section's first page.
//!
//! The interface never waits: builds run on a worker thread whenever the book or the edition
//! changes, and page images are drawn on another, only for the pages in view, then cached.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use jcj_i18n::{tr, tr_with};
use jcj_shell::egui;
use wordsmith_book_check::build::{self, Print};
use wordsmith_book_model::Edition;

use crate::doc::OpenBook;

/// A finished build.
pub struct Built {
    /// The project revision it was built from.
    pub revision: u64,
    /// The edition it shows.
    pub edition: String,
    /// The laid-out edition.
    pub print: Print,
    /// How long it took from the request.
    pub took: Duration,
    /// Only the opening (front matter and first chapter); the whole book follows.
    pub opening: bool,
}

/// A page image drawn for the preview, at a resolution.
struct Drawn {
    revision: u64,
    opening: bool,
    page: usize,
    ppi: u32,
    image: Option<Pixels>,
}

/// RGBA pixels and their size.
struct Pixels {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

enum Job {
    Build(u64, String),
}

/// What a build worker sends: the opening first, then the whole book.
type Stage = Result<Built, String>;

/// What the preview shows and the work it has in hand.
pub struct PrintPreview {
    /// The print edition shown (its id); the first print edition when none is chosen.
    pub edition: Option<String>,
    /// Zoom: interface points per point of the page.
    pub zoom: f32,
    /// Fit a spread to the panel's width (until the zoom is changed by hand).
    pub fit: bool,
    /// Show trim, margin and bleed guides.
    pub guides: bool,
    /// A page to bring into view on the next frame (from 0).
    pub scroll_to: Option<usize>,
    /// A piece to bring into view once the build has it.
    pub target: Option<String>,
    /// The latest finished build.
    pub built: Option<Arc<Built>>,
    /// Why the last build failed, if it did.
    pub error: Option<String>,
    job: Option<(Job, Receiver<Stage>)>,
    renderer: Option<(Sender<(Arc<Built>, usize, u32)>, Receiver<Drawn>)>,
    requested: BTreeSet<(u64, bool, usize, u32)>,
    textures: BTreeMap<(usize, u32), egui::TextureHandle>,
    /// The revision, edition and stage the textures belong to.
    texture_key: (u64, String, bool),
}

impl Default for PrintPreview {
    fn default() -> Self {
        PrintPreview {
            edition: None,
            zoom: 0.6,
            fit: true,
            guides: true,
            scroll_to: None,
            target: None,
            built: None,
            error: None,
            job: None,
            renderer: None,
            requested: BTreeSet::new(),
            textures: BTreeMap::new(),
            texture_key: (u64::MAX, String::new(), false),
        }
    }
}

/// The print editions of a project.
pub fn print_editions(b: &OpenBook) -> Vec<Edition> {
    b.files.project.editions.iter().filter(|e| e.format.is_print()).cloned().collect()
}

/// The spreads of a book of `pages` pages: the first page alone on the right, then pairs.
pub fn spreads(pages: usize) -> Vec<(Option<usize>, Option<usize>)> {
    if pages == 0 {
        return Vec::new();
    }
    let mut out = vec![(None, Some(0))];
    let mut i = 1;
    while i < pages {
        out.push((Some(i), (i + 1 < pages).then_some(i + 1)));
        i += 2;
    }
    out
}

/// The spread that holds page `index`.
pub fn spread_of(index: usize) -> usize {
    index.div_ceil(2)
}

impl PrintPreview {
    /// The edition shown.
    pub fn edition_of(&self, b: &OpenBook) -> Option<Edition> {
        let all = print_editions(b);
        self.edition.as_ref().and_then(|id| all.iter().find(|e| &e.id == id).cloned()).or_else(|| all.into_iter().next())
    }

    /// Whether a build is running.
    pub fn building(&self) -> bool {
        self.job.is_some()
    }

    /// Start a build if the book or the edition changed since the last one (and none runs).
    pub fn update(&mut self, b: &OpenBook) {
        self.poll();
        let Some(edition) = self.edition_of(b) else { return };
        let current = self.built.as_ref().is_some_and(|x| x.revision == b.revision && x.edition == edition.id && !x.opening);
        let running = matches!(&self.job, Some((Job::Build(r, e), _)) if *r == b.revision && *e == edition.id);
        if current || running || self.job.is_some() {
            return;
        }
        let Ok(src) = &b.source else { return };
        // Everything the build needs, copied, so the worker owns it.
        let project = b.files.project.clone();
        let designs = b.files.designs.clone();
        let book = wordsmith_book_format::fixes::book_for_build(&project.fixes, src);
        let pictures: BTreeMap<String, Vec<u8>> = pictures_of(&book).into_iter().filter_map(|p| b.picture(&p).map(|bytes| (p, bytes))).collect();
        let (tx, rx) = channel();
        let revision = b.revision;
        let id = edition.id.clone();
        std::thread::spawn(move || {
            let start = Instant::now();
            let images = |p: &str| pictures.get(p).cloned();
            // The opening first, so the first pages show at once; then the whole book.
            if let Ok(print) = build::print_opening(&project, &designs, &book, &edition, &images)
                && tx.send(Ok(Built { revision, edition: edition.id.clone(), print, took: start.elapsed(), opening: true })).is_err()
            {
                return;
            }
            let out = build::print(&project, &designs, &book, &edition, &images).map(|print| Built {
                revision,
                edition: edition.id.clone(),
                print,
                took: start.elapsed(),
                opening: false,
            });
            let _ = tx.send(out);
        });
        self.job = Some((Job::Build(revision, id), rx));
    }

    /// Take a finished build, if there is one.
    pub fn poll(&mut self) {
        while let Some((_, rx)) = &self.job {
            match rx.try_recv() {
                Ok(Ok(built)) => {
                    let done = !built.opening;
                    self.built = Some(Arc::new(built));
                    self.error = None;
                    if done {
                        self.job = None;
                    }
                }
                Ok(Err(e)) => {
                    self.error = Some(e);
                    self.job = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.error = Some("the build stopped".into());
                    self.job = None;
                }
            }
        }
    }

    /// Wait for the running build (tests and agent tools); gives whether one finished.
    pub fn wait(&mut self, limit: Duration) -> bool {
        let start = Instant::now();
        while self.job.is_some() && start.elapsed() < limit {
            self.poll();
            std::thread::sleep(Duration::from_millis(5));
        }
        self.job.is_none()
    }

    /// Whether page `index` of the latest build is drawn and ready to show.
    pub fn page_ready(&self, index: usize) -> bool {
        self.built.as_ref().is_some_and(|b| self.texture_key.0 == b.revision && self.texture_key.2 == b.opening)
            && self.textures.range((index, 0)..=(index, u32::MAX)).next().is_some()
    }

    /// The first page of a piece (a section id or `element:<id>`), from 0.
    pub fn page_of(&self, piece: &str) -> Option<usize> {
        let built = self.built.as_ref()?;
        built.print.typeset.page_map().into_iter().find(|s| s.id == piece).map(|s| s.page.saturating_sub(1))
    }

    fn renderer(&mut self) -> &(Sender<(Arc<Built>, usize, u32)>, Receiver<Drawn>) {
        self.renderer.get_or_insert_with(|| {
            let (tx, rx) = channel::<(Arc<Built>, usize, u32)>();
            let (back, out) = channel();
            std::thread::spawn(move || {
                while let Ok((built, page, ppi)) = rx.recv() {
                    #[allow(clippy::cast_precision_loss, reason = "a resolution in pixels per inch")]
                    let image =
                        built.print.typeset.page_image(page, ppi as f32).ok().map(|i| Pixels { width: i.width, height: i.height, pixels: i.pixels });
                    if back.send(Drawn { revision: built.revision, opening: built.opening, page, ppi, image }).is_err() {
                        break;
                    }
                }
            });
            (tx, out)
        })
    }

    /// Take drawn pages and turn them into textures.
    fn receive(&mut self, ctx: &egui::Context) {
        let Some(built) = self.built.clone() else { return };
        let key = (built.revision, built.edition.clone(), built.opening);
        if self.texture_key != key {
            self.textures.clear();
            self.requested.clear();
            self.texture_key = key;
        }
        let drawn: Vec<Drawn> = match &self.renderer {
            Some((_, rx)) => rx.try_iter().collect(),
            None => return,
        };
        for d in drawn {
            if d.revision != built.revision || d.opening != built.opening {
                continue;
            }
            if let Some(img) = d.image {
                let size = [usize::try_from(img.width).unwrap_or(0), usize::try_from(img.height).unwrap_or(0)];
                let color = egui::ColorImage::from_rgba_unmultiplied(size, &img.pixels);
                let tex = ctx.load_texture(format!("print-page-{}-{}", d.page, d.ppi), color, egui::TextureOptions::LINEAR);
                self.textures.insert((d.page, d.ppi), tex);
            }
        }
    }

    fn request(&mut self, page: usize, ppi: u32) {
        let Some(built) = self.built.clone() else { return };
        if self.textures.contains_key(&(page, ppi)) || !self.requested.insert((built.revision, built.opening, page, ppi)) {
            return;
        }
        let _ = self.renderer().0.send((built, page, ppi));
    }

    /// The best texture for a page: at `ppi`, else any resolution drawn.
    fn texture(&self, page: usize, ppi: u32) -> Option<&egui::TextureHandle> {
        self.textures.get(&(page, ppi)).or_else(|| self.textures.range((page, 0)..=(page, u32::MAX)).next_back().map(|(_, t)| t))
    }

    /// Draw the preview; gives the piece whose page was clicked.
    pub fn ui(&mut self, ui: &mut egui::Ui, b: &OpenBook) -> Option<String> {
        self.update(b);
        self.receive(ui.ctx());
        let editions = print_editions(b);
        let shown = self.edition_of(b);
        ui.horizontal_wrapped(|ui| {
            if editions.len() > 1 {
                for e in &editions {
                    if ui.selectable_label(shown.as_ref().is_some_and(|s| s.id == e.id), &e.name).clicked() {
                        self.edition = Some(e.id.clone());
                    }
                }
            }
            ui.checkbox(&mut self.guides, tr("Guides"));
            if ui.button(tr("Zoom out")).clicked() {
                self.zoom = (self.zoom / 1.25).max(0.1);
                self.fit = false;
            }
            if ui.button(tr("Zoom in")).clicked() {
                self.zoom = (self.zoom * 1.25).min(3.0);
                self.fit = false;
            }
            if ui.selectable_label(self.fit, tr("Fit")).clicked() {
                self.fit = true;
            }
            ui.label(format!("{:.0}%", self.zoom * 100.0));
        });
        if shown.is_none() {
            ui.label(tr("The book has no print edition."));
            return None;
        }
        if self.building() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(tr("Laying out the book…"));
            });
            ui.ctx().request_repaint_after(Duration::from_millis(50));
        }
        if let Some(e) = &self.error {
            ui.colored_label(ui.visuals().error_fg_color, tr_with("The preview could not be built: {why}", &[("why", &e.to_string())]));
        }
        let built = self.built.clone()?;
        if !self.building()
            && let Some(piece) = self.target.take()
        {
            self.scroll_to = self.page_of(&piece);
        }
        let t = &built.print.typeset;
        let pages = t.page_count();
        if built.opening {
            ui.label(tr("The opening pages; laying out the rest of the book…"));
        } else {
            ui.label(format!("{pages} pages · gutter {:.3} in · built in {:.1} s", t.gutter(), built.took.as_secs_f32()));
        }
        let (pw, ph) = t.page_size(0).unwrap_or((432.0, 648.0));
        let bleed = if self.guides { built.print.profile.bleed * 72.0 } else { 0.0 };
        if self.fit {
            let room = ui.available_width() - ui.spacing().item_spacing.x - 2.0 * (bleed + 4.0) - 16.0;
            self.zoom = (room / (2.0 * pw)).clamp(0.1, 3.0);
        }
        let zoom = self.zoom;
        let gap = 24.0;
        let row_h = ph * zoom + gap;
        let rows = spreads(pages);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a resolution, a few hundred at most")]
        let ppi = ((72.0 * zoom * ui.ctx().pixels_per_point()) / 24.0).ceil().max(1.0) as u32 * 24;
        let mut clicked = None;
        let mut area = egui::ScrollArea::both().auto_shrink([false, false]).id_salt("print-preview");
        if let Some(p) = self.scroll_to.take() {
            #[allow(clippy::cast_precision_loss, reason = "a spread index")]
            let y = spread_of(p) as f32 * row_h;
            area = area.vertical_scroll_offset(y);
        }
        let mut wanted = Vec::new();
        area.show_rows(ui, row_h, rows.len(), |ui, range| {
            for r in range {
                let (left, right) = rows[r];
                ui.horizontal(|ui| {
                    for page in [left, right] {
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(pw * zoom, ph * zoom), egui::Sense::click());
                        let Some(i) = page else { continue };
                        wanted.push(i);
                        let painter = ui.painter_at(rect.expand(bleed * zoom + 2.0));
                        painter.rect_filled(rect, 0.0, egui::Color32::WHITE);
                        match self.texture(i, ppi) {
                            Some(tex) => {
                                painter.image(
                                    tex.id(),
                                    rect,
                                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                    egui::Color32::WHITE,
                                );
                            }
                            None => {
                                painter.text(rect.center(), egui::Align2::CENTER_CENTER, "…", egui::FontId::proportional(14.0), egui::Color32::GRAY);
                            }
                        }
                        if self.guides {
                            // A page that bleeds is drawn whole; its trim is inside it.
                            let trim = (t.bleed() > 0.0).then(|| {
                                let (x0, y0, x1, y1) = t.trim_box(i);
                                egui::Rect::from_min_max(
                                    rect.min + egui::vec2(x0 * zoom, (ph - y1) * zoom),
                                    rect.min + egui::vec2(x1 * zoom, (ph - y0) * zoom),
                                )
                            });
                            guides(&painter, rect, trim, zoom, bleed, t.body_area(i));
                        }
                        let folio = t.folio(i);
                        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tr_with("Page {n}", &[("n", &(i + 1).to_string())])));
                        let resp =
                            resp.on_hover_text(tr_with("Page {n} ({folio}): open its section", &[("n", &(i + 1).to_string()), ("folio", &folio)]));
                        if resp.clicked() {
                            clicked = t.section_of_page(i);
                        }
                    }
                });
                ui.add_space(gap - ui.spacing().item_spacing.y);
            }
        });
        for i in wanted {
            self.request(i, ppi);
        }
        if self.renderer.is_some() && self.requested.len() > self.textures.len() {
            ui.ctx().request_repaint_after(Duration::from_millis(30));
        }
        clicked
    }
}

/// Trim (the page's edge), margin (the body area) and bleed guides.
fn guides(painter: &egui::Painter, page: egui::Rect, trim_rect: Option<egui::Rect>, zoom: f32, bleed: f32, body: Option<(f32, f32, f32, f32)>) {
    let trim = egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 40, 40));
    let margin = egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(40, 110, 200, 140));
    let bleed_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(200, 40, 40, 160));
    match trim_rect {
        // The page bleeds: its edge is the bleed, the trim is inside.
        Some(t) => {
            painter.rect_stroke(page, 0.0, bleed_stroke, egui::StrokeKind::Inside);
            painter.rect_stroke(t, 0.0, trim, egui::StrokeKind::Middle);
        }
        None => {
            painter.rect_stroke(page, 0.0, trim, egui::StrokeKind::Inside);
        }
    }
    if let Some((x, y, w, h)) = body {
        let r = egui::Rect::from_min_size(page.min + egui::vec2(x * zoom, y * zoom), egui::vec2(w * zoom, h * zoom));
        painter.rect_stroke(r, 0.0, margin, egui::StrokeKind::Middle);
    }
    if bleed > 0.0 && trim_rect.is_none() {
        painter.rect_stroke(page.expand(bleed * zoom), 0.0, bleed_stroke, egui::StrokeKind::Middle);
    }
}

/// Every picture path a book names, in order, once each.
pub fn pictures_of(book: &wordsmith_manuscript_format::book::Book) -> Vec<String> {
    fn walk(s: &wordsmith_manuscript_format::book::Section, out: &mut Vec<String>) {
        for b in &s.content {
            if let wordsmith_manuscript_model::Block::Image { src, .. } = b
                && !out.contains(src)
            {
                out.push(src.clone());
            }
        }
        for c in &s.children {
            walk(c, out);
        }
    }
    let mut out = Vec::new();
    for s in book.front.iter().chain(&book.body).chain(&book.back) {
        walk(s, &mut out);
    }
    out
}
