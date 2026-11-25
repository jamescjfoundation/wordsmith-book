// SPDX-License-Identifier: MIT OR Apache-2.0
//! The printed cover (PRT-9, PRT-10): the full-wrap template for a print edition (back, spine and
//! front with bleed, safe areas, the spine text rule and the barcode area), as a PDF and a PNG
//! guide; and a supplied cover, PDF or image, checked against it.
//!
//! The cover's art is made elsewhere; Book gives its exact size and checks what comes back.

use std::fmt::Write as _;

use wordsmith_book_model::{Paper, Severity};

use crate::preflight::{Finding, Link};
use crate::profile::Profile;

/// How far inside the trim text and pictures that matter must stay, inches.
pub const SAFE: f64 = 0.125;
/// Space kept clear on each side of the spine's text, inches.
pub const SPINE_SAFE: f64 = 0.0625;
/// The resolution a cover's pictures need, pixels per inch.
pub const MIN_PPI: f64 = 300.0;

/// A rectangle on the cover, inches from its bottom left.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Area {
    /// Left.
    pub x: f64,
    /// Bottom.
    pub y: f64,
    /// Width.
    pub w: f64,
    /// Height.
    pub h: f64,
}

impl Area {
    fn contains(&self, o: &Area) -> bool {
        o.x >= self.x - 1e-9 && o.y >= self.y - 1e-9 && o.x + o.w <= self.x + self.w + 1e-9 && o.y + o.h <= self.y + self.h + 1e-9
    }
}

/// A full-wrap cover's geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct Template {
    /// The whole sheet, bleed included, inches.
    pub width: f64,
    /// Its height, inches.
    pub height: f64,
    /// The bleed on every outside edge, inches.
    pub bleed: f64,
    /// The spine's width, inches.
    pub spine: f64,
    /// The page count it is for.
    pub pages: usize,
    /// The back cover (trimmed).
    pub back: Area,
    /// The spine.
    pub spine_area: Area,
    /// The front cover (trimmed).
    pub front: Area,
    /// Where the back cover's text and pictures stay.
    pub back_safe: Area,
    /// Where the front cover's text and pictures stay.
    pub front_safe: Area,
    /// Where the spine's text stays, when the book is thick enough for any.
    pub spine_safe: Option<Area>,
    /// Kept clear for the barcode, on the back cover's lower right.
    pub barcode: Area,
}

/// The template for a print edition of `pages` pages on `paper`, at trim (`w`, `h`) inches.
pub fn template(profile: &Profile, trim: (f64, f64), pages: usize, paper: Paper) -> Template {
    let (tw, th) = trim;
    let bleed = f64::from(profile.bleed);
    let spine = profile.spine(pages, paper);
    let width = 2.0 * bleed + 2.0 * tw + spine;
    let height = 2.0 * bleed + th;
    let back = Area { x: bleed, y: bleed, w: tw, h: th };
    let spine_area = Area { x: bleed + tw, y: bleed, w: spine, h: th };
    let front = Area { x: bleed + tw + spine, y: bleed, w: tw, h: th };
    let inset = |a: &Area, by: f64| Area { x: a.x + by, y: a.y + by, w: (a.w - 2.0 * by).max(0.0), h: (a.h - 2.0 * by).max(0.0) };
    let back_safe = inset(&back, SAFE);
    let front_safe = inset(&front, SAFE);
    let spine_safe = (pages >= profile.spine_text_min_pages).then(|| Area {
        x: spine_area.x + SPINE_SAFE,
        y: spine_area.y + SAFE,
        w: (spine - 2.0 * SPINE_SAFE).max(0.0),
        h: th - 2.0 * SAFE,
    });
    let (bw, bh) = (f64::from(profile.barcode[0]), f64::from(profile.barcode[1]));
    let barcode = Area { x: back_safe.x + back_safe.w - bw, y: back_safe.y, w: bw, h: bh };
    Template { width, height, bleed, spine, pages, back, spine_area, front, back_safe, front_safe, spine_safe, barcode }
}

fn num(v: f64) -> String {
    let s = format!("{v:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// The template as a one-page PDF guide: the trim lines, the spine, the safe areas and the
/// barcode area, in points. The same template gives the same bytes.
pub fn template_pdf(t: &Template) -> Vec<u8> {
    let pt = |v: f64| num(v * 72.0);
    let rect = |a: &Area| format!("{} {} {} {} re S\n", pt(a.x), pt(a.y), pt(a.w), pt(a.h));
    let mut c = String::new();
    // The sheet's edge (bleed) in red, the trim and spine in black, safe areas in blue, the
    // barcode area in green.
    let _ = write!(c, "0.8 0 0 RG 0.5 w {}", rect(&Area { x: 0.0, y: 0.0, w: t.width, h: t.height }));
    let _ = write!(c, "0 0 0 RG 0.75 w {}{}{}", rect(&t.back), rect(&t.spine_area), rect(&t.front));
    let _ = write!(c, "0 0.3 0.8 RG 0.5 w [4 3] 0 d {}{}", rect(&t.back_safe), rect(&t.front_safe));
    if let Some(s) = &t.spine_safe {
        c.push_str(&rect(s));
    }
    let _ = write!(c, "[] 0 d 0 0.55 0.2 RG 1 w {}", rect(&t.barcode));
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
        format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Contents 4 0 R /Resources << >> >>", pt(t.width), pt(t.height)),
        format!("<< /Length {} >>\nstream\n{c}endstream", c.len()),
    ];
    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (i, o) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{o}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
    for o in offsets {
        out.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1).as_bytes());
    out
}

/// The template as a PNG guide at `ppi` pixels per inch (the same lines as the PDF).
pub fn template_png(t: &Template, ppi: f64) -> Vec<u8> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a cover guide, a few thousand pixels")]
    let px = |v: f64| (v * ppi).round().max(0.0) as u32;
    let (w, h) = (px(t.width).max(1), px(t.height).max(1));
    let mut img = image::RgbImage::from_pixel(w, h, image::Rgb([255, 255, 255]));
    let mut frame = |a: &Area, color: [u8; 3]| {
        let (x0, x1) = (px(a.x).min(w - 1), px(a.x + a.w).min(w - 1));
        // Image rows run from the top.
        let (y0, y1) = (h - 1 - px(a.y + a.h).min(h - 1), h - 1 - px(a.y).min(h - 1));
        for x in x0..=x1 {
            img.put_pixel(x, y0, image::Rgb(color));
            img.put_pixel(x, y1, image::Rgb(color));
        }
        for y in y0..=y1 {
            img.put_pixel(x0, y, image::Rgb(color));
            img.put_pixel(x1, y, image::Rgb(color));
        }
    };
    frame(&Area { x: 0.0, y: 0.0, w: t.width, h: t.height }, [204, 0, 0]);
    for a in [&t.back, &t.spine_area, &t.front] {
        frame(a, [0, 0, 0]);
    }
    for a in [&t.back_safe, &t.front_safe] {
        frame(a, [0, 77, 204]);
    }
    if let Some(s) = &t.spine_safe {
        frame(s, [0, 77, 204]);
    }
    frame(&t.barcode, [0, 140, 51]);
    let mut out = std::io::Cursor::new(Vec::new());
    let _ = img.write_to(&mut out, image::ImageFormat::Png);
    out.into_inner()
}

fn finding(severity: Severity, message: String) -> Finding {
    Finding { rule: "cover", severity, message, link: Link::None }
}

/// What is wrong with a supplied cover (a PDF, or a PNG or JPEG image) for this template.
pub fn check(bytes: &[u8], t: &Template) -> Vec<Finding> {
    let near = |a: f64, b: f64| (a - b).abs() <= 0.01;
    let size = format!("{:.3} × {:.3} in", t.width, t.height);
    let mut out = Vec::new();
    if bytes.starts_with(b"%PDF") {
        let limits = jcj_output::ReadLimits { max_input_bytes: 512 << 20, max_pages: 10, max_glyphs_per_page: 1 };
        let facts = match jcj_output::read_facts(bytes, &limits) {
            Ok(f) => f,
            Err(e) => return vec![finding(Severity::Blocks, format!("The cover PDF could not be read: {e}."))],
        };
        if facts.pages.len() != 1 {
            out.push(finding(Severity::Blocks, format!("The cover PDF has {} pages; it needs one.", facts.pages.len())));
        }
        if let Some(p) = facts.pages.first() {
            let (w, h) = (p.media.width() / 72.0, p.media.height() / 72.0);
            if !near(w, t.width) || !near(h, t.height) {
                out.push(finding(Severity::Blocks, format!("The cover is {w:.3} × {h:.3} in; for {} pages it must be {size}.", t.pages)));
            }
            for im in p.images.iter().filter(|i| !i.mask && i.ppi() < MIN_PPI) {
                out.push(finding(
                    Severity::Warning,
                    format!("A picture on the cover prints at {:.0} pixels per inch; 300 or more prints sharply.", im.ppi()),
                ));
            }
            if !p.annotations.is_empty() {
                out.push(finding(Severity::Blocks, "The cover PDF has links or comments; it must have none.".into()));
            }
        }
        for f in facts.unembedded_fonts() {
            out.push(finding(Severity::Blocks, format!("The font {} is not embedded in the cover PDF.", f.name)));
        }
        return out;
    }
    let Some((pw, ph)) = wordsmith_book_model::picture::size(bytes) else {
        return vec![finding(Severity::Blocks, "The cover is not a PDF, PNG or JPEG file Book can read.".into())];
    };
    let (pw, ph) = (f64::from(pw), f64::from(ph));
    let (ppi_w, ppi_h) = (pw / t.width, ph / t.height);
    if (ppi_w / ppi_h - 1.0).abs() > 0.01 {
        out.push(finding(
            Severity::Blocks,
            format!("The cover image is {pw:.0} × {ph:.0} pixels, not the shape of the {size} cover for {} pages.", t.pages),
        ));
    }
    if ppi_w.min(ppi_h) < MIN_PPI - 0.5 {
        out.push(finding(
            Severity::Blocks,
            format!(
                "The cover image is {:.0} pixels per inch at {size}; it needs {MIN_PPI:.0} ({:.0} × {:.0} pixels).",
                ppi_w.min(ppi_h),
                (t.width * MIN_PPI).ceil(),
                (t.height * MIN_PPI).ceil()
            ),
        ));
    }
    out
}

/// Whether the barcode area sits inside the back cover's safe area (a template check).
pub fn barcode_fits(t: &Template) -> bool {
    t.back_safe.contains(&t.barcode)
}
