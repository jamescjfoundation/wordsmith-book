// SPDX-License-Identifier: MIT OR Apache-2.0
//! The preflight panel (PRE-1): run preflight for an edition in the background, then list what
//! it found, blocking findings first, each with a way to its page, section or edition.

use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use jcj_shell::egui;
use serde_json::{Value, json};
use wordsmith_book_check::preflight::{Finding, Link, Report};
use wordsmith_book_model::Severity;

use crate::doc::OpenBook;

/// Where the user asked to go from a finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GoTo {
    /// A page of the print preview, from 1.
    Page(usize),
    /// A piece of the book.
    Section(String),
    /// The details panel, for an edition.
    Edition(String),
}

/// The panel's state.
#[derive(Default)]
pub struct Panel {
    /// The edition to check (its id); the first edition when none is chosen.
    pub edition: Option<String>,
    /// The last report, and the project revision it was made for.
    pub report: Option<(u64, Report)>,
    job: Option<Receiver<Report>>,
    pending_revision: u64,
}

/// A finding as JSON, for agents.
pub fn finding_json(f: &Finding) -> Value {
    let link = match &f.link {
        Link::None => Value::Null,
        Link::Page(n) => json!({ "page": n }),
        Link::Section(s) => json!({ "section": s }),
        Link::Edition(e) => json!({ "edition": e }),
        Link::File(p) => json!({ "file": p }),
    };
    json!({ "rule": f.rule, "severity": f.severity.title(), "message": f.message, "link": link })
}

/// Run preflight for an edition of the open book (on this thread).
pub fn run(b: &OpenBook, edition: &str) -> Result<Report, String> {
    let src = b.source.as_ref().map_err(Clone::clone)?;
    let e = b.files.project.editions.iter().find(|e| e.id == edition).ok_or_else(|| format!("there is no edition `{edition}`"))?;
    Ok(wordsmith_book_check::preflight::preflight(&b.files.project, &b.files.designs, src, e, &|p| b.picture(p)).0)
}

impl Panel {
    /// Whether preflight is running.
    pub fn running(&self) -> bool {
        self.job.is_some()
    }

    /// Start preflight for an edition on a worker thread.
    pub fn start(&mut self, b: &OpenBook, edition: &str) {
        let Ok(src) = &b.source else { return };
        let Some(e) = b.files.project.editions.iter().find(|e| e.id == edition).cloned() else { return };
        let project = b.files.project.clone();
        let designs = b.files.designs.clone();
        let src = src.clone();
        let book = wordsmith_book_format::fixes::book_for_build(&project.fixes, &src);
        let pictures: std::collections::BTreeMap<String, Vec<u8>> = crate::preview::pictures_of(&book)
            .into_iter()
            .chain(std::iter::once(e.cover.clone()).filter(|c| !c.is_empty()))
            .filter_map(|p| b.picture(&p).map(|x| (p, x)))
            .collect();
        let revision = b.revision;
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let (r, _) = wordsmith_book_check::preflight::preflight(&project, &designs, &src, &e, &|p| pictures.get(p).cloned());
            let _ = tx.send(r);
        });
        self.job = Some(rx);
        self.report = self.report.take().filter(|(rev, _)| *rev == revision);
        self.pending_revision = revision;
    }

    /// Take a finished report.
    pub fn poll(&mut self) {
        if let Some(rx) = &self.job
            && let Ok(r) = rx.try_recv()
        {
            self.report = Some((self.pending_revision, r));
            self.job = None;
        }
    }

    /// Wait for the running preflight (tests and agents).
    pub fn wait(&mut self, limit: Duration) -> bool {
        let start = std::time::Instant::now();
        while self.job.is_some() && start.elapsed() < limit {
            self.poll();
            std::thread::sleep(Duration::from_millis(5));
        }
        self.job.is_none()
    }

    /// Draw the panel; gives where to go when a finding's link is followed.
    pub fn ui(&mut self, ui: &mut egui::Ui, b: &OpenBook) -> Option<GoTo> {
        self.poll();
        let editions = b.files.project.editions.clone();
        let chosen = self.edition.clone().filter(|id| editions.iter().any(|e| &e.id == id)).or_else(|| editions.first().map(|e| e.id.clone()));
        let mut go = None;
        ui.horizontal_wrapped(|ui| {
            for e in &editions {
                if ui.selectable_label(chosen.as_deref() == Some(e.id.as_str()), &e.name).clicked() {
                    self.edition = Some(e.id.clone());
                }
            }
        });
        let Some(edition) = chosen else {
            ui.label("The book has no editions to check.");
            return None;
        };
        if ui.add_enabled(!self.running(), egui::Button::new("Run Preflight")).clicked() {
            self.start(b, &edition);
        }
        if self.running() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Building and checking the edition…");
            });
            ui.ctx().request_repaint_after(Duration::from_millis(50));
        }
        let Some((rev, report)) = &self.report else { return None };
        if *rev != b.revision {
            ui.label(egui::RichText::new("The book changed since this check; run it again.").italics());
        }
        let blocks = report.of(Severity::Blocks).len();
        let summary = if blocks == 0 {
            "Nothing blocks export.".to_owned()
        } else {
            format!("{blocks} problem{} must be fixed before export.", if blocks == 1 { "" } else { "s" })
        };
        ui.strong(summary);
        egui::ScrollArea::vertical().id_salt("preflight").show(ui, |ui| {
            for f in &report.findings {
                ui.horizontal_wrapped(|ui| {
                    let color = match f.severity {
                        Severity::Blocks => ui.visuals().error_fg_color,
                        Severity::Warning => ui.visuals().warn_fg_color,
                        Severity::Note => ui.visuals().weak_text_color(),
                    };
                    ui.colored_label(color, f.severity.title());
                    ui.label(&f.message);
                    match &f.link {
                        Link::Page(n) => {
                            if ui.small_button(format!("Go to Page {n}")).clicked() {
                                go = Some(GoTo::Page(*n));
                            }
                        }
                        Link::Section(s) => {
                            if ui.small_button("Open Section").clicked() {
                                go = Some(GoTo::Section(s.clone()));
                            }
                        }
                        Link::Edition(e) => {
                            if ui.small_button("Open Edition Details").clicked() {
                                go = Some(GoTo::Edition(e.clone()));
                            }
                        }
                        Link::File(_) | Link::None => {}
                    }
                });
                ui.add_space(4.0);
            }
        });
        go
    }
}
