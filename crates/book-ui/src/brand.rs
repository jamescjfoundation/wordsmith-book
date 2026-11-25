// SPDX-License-Identifier: MIT OR Apache-2.0
//! Wordsmith Book's brand (`assets/brand/`, from the Wordsmith brand kit): its design tokens, its
//! app icon and its mark. The brand lives here, never in the core.
//!
//! The mark is the letter B followed by the ribbon cursor, as
//! `assets/brand/wordsmith-book-mark-ink.svg`. The B is drawn from a white mask
//! (`assets/brand/mark-mask.png`, made by `tools/brand-mask.py`) tinted at paint time, so one
//! picture serves every theme; the ribbon is plain shapes in Ribbon Green.

use jcj_shell::egui::{self, Color32, Context, Id, Painter, Rect, Shape, Stroke, TextureHandle, TextureOptions, pos2, vec2};
use jcj_shell::theme::Tokens;

/// The tokens file, as shipped.
pub const TOKENS_JSON: &str = include_str!("../../../assets/brand/tokens.json");

/// The app icon at 256 pixels (PNG), for the window.
pub const ICON_256: &[u8] = include_bytes!("../../../assets/brand/icon-256.png");

const MARK_MASK: &[u8] = include_bytes!("../../../assets/brand/mark-mask.png");

/// Ribbon Green: the ribbon cursor in the mark, and small accents.
pub const RIBBON_GREEN: Color32 = Color32::from_rgb(0x2E, 0x94, 0x73);

/// The mark's view box, in the SVG's units: width and height.
const MARK_SIZE: (f32, f32) = (313.12, 351.88);
/// The ribbon in the same units: left, right, the bottom of its sides, the notch's point.
const RIBBON: (f32, f32, f32, f32) = (251.84, 313.04, 351.84, 318.18);

/// The name of the fallback font that draws Book's ornaments and typographic marks in the
/// window (the interface fonts lack fleurons).
pub const ORNAMENTS: &str = "book-ornaments";

/// Make sure the window can draw ornaments: Book's classic serif as the last fallback.
pub fn ensure_fonts(ctx: &jcj_shell::egui::Context) {
    use jcj_shell::egui::epaint::text::{FontData, FontInsert, FontPriority, InsertFontFamily};
    if ctx.fonts(|f| f.definitions().font_data.contains_key(ORNAMENTS)) {
        return;
    }
    let Some(face) = wordsmith_book_design::fonts::regular(wordsmith_book_design::fonts::Role::ClassicSerif) else { return };
    ctx.add_font(FontInsert {
        name: ORNAMENTS.into(),
        data: FontData::from_static(face.bytes),
        families: vec![
            InsertFontFamily { family: jcj_shell::egui::FontFamily::Proportional, priority: FontPriority::Lowest },
            InsertFontFamily { family: jcj_shell::egui::FontFamily::Monospace, priority: FontPriority::Lowest },
        ],
    });
}

/// Book's tokens; the neutral set if the shipped file could not be read (a test guards it).
pub fn tokens() -> Tokens {
    Tokens::from_json(TOKENS_JSON).unwrap_or_default()
}

/// The mask's texture, loaded once per context.
fn mask(ctx: &Context) -> Option<TextureHandle> {
    let id = Id::unique("wordsmith-book-mark-mask");
    if let Some(tex) = ctx.data(|d| d.get_temp::<Option<TextureHandle>>(id)) {
        return tex;
    }
    let tex = image::load_from_memory_with_format(MARK_MASK, image::ImageFormat::Png).ok().map(|img| {
        let img = img.into_rgba8();
        let size = [img.width() as usize, img.height() as usize];
        // White with the mask's coverage, so a tint paints it in any color.
        let pixels = img.pixels().map(|p| Color32::from_white_alpha(p[3])).collect();
        ctx.load_texture("brand-mark", egui::ColorImage::new(size, pixels), TextureOptions::LINEAR)
    });
    ctx.data_mut(|d| d.insert_temp(id, tex.clone()));
    tex
}

/// The mark's width at `height` points.
pub fn mark_width(height: f32) -> f32 {
    height * MARK_SIZE.0 / MARK_SIZE.1
}

/// Paint the mark (the B in `ink` and the ribbon in Ribbon Green) centered in `rect`, as the shell
/// asks for it in the quick-access row and the About page.
pub fn paint_mark(painter: &Painter, rect: Rect, ink: Color32) {
    let s = (rect.width() / MARK_SIZE.0).min(rect.height() / MARK_SIZE.1);
    let min = rect.center() - vec2(MARK_SIZE.0, MARK_SIZE.1) * s / 2.0;
    let at = |x: f32, y: f32| pos2(min.x + x * s, min.y + y * s);
    if let Some(tex) = mask(painter.ctx()) {
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        painter.image(tex.id(), Rect::from_min_max(at(0.0, 0.0), at(MARK_SIZE.0, MARK_SIZE.1)), uv, ink);
    }
    // The ribbon: a band down to the notch, then the two tails either side of the V.
    let (left, right, bottom, notch) = RIBBON;
    let mid = (left + right) / 2.0;
    painter.rect_filled(Rect::from_min_max(at(left, 0.0), at(right, notch)), 0.0, RIBBON_GREEN);
    for tail in [[at(left, notch), at(mid, notch), at(left, bottom)], [at(mid, notch), at(right, notch), at(right, bottom)]] {
        painter.add(Shape::convex_polygon(tail.to_vec(), RIBBON_GREEN, Stroke::NONE));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jcj_shell::theme::Mode;

    #[test]
    fn the_shipped_tokens_read_and_differ_from_the_neutral_set() {
        let t = Tokens::from_json(TOKENS_JSON).expect("tokens.json reads");
        let neutral = Tokens::default();
        for mode in MODES {
            assert_ne!(t.colors(mode), neutral.colors(mode), "{mode:?} has Book's own colors");
        }
    }

    const INK: Color32 = Color32::from_rgb(0x1C, 0x23, 0x40);
    const PAPER: Color32 = Color32::from_rgb(0xFA, 0xFA, 0xF7);
    const MARGIN_BLUE: Color32 = Color32::from_rgb(0x3B, 0x4F, 0xA8);
    const SLATE: Color32 = Color32::from_rgb(0x5A, 0x60, 0x80);
    const NIGHT: Color32 = Color32::from_rgb(0x12, 0x16, 0x2B);
    const MODES: [Mode; 3] = [Mode::Light, Mode::Dark, Mode::ExtraDark];

    /// WCAG 2.x relative luminance.
    fn luminance(c: Color32) -> f32 {
        let lin = |v: u8| {
            let v = f32::from(v) / 255.0;
            if v <= 0.040_45 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
        };
        0.2126 * lin(c.r()) + 0.7152 * lin(c.g()) + 0.0722 * lin(c.b())
    }

    /// WCAG 2.x contrast ratio, 1 to 21.
    fn contrast(a: Color32, b: Color32) -> f32 {
        let (x, y) = (luminance(a), luminance(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    /// The light theme is the brand kit's palette: Iron Gall Ink on Page White, Slate for
    /// secondary text, Margin Blue for links and focus, ink for the brand fill, and Ribbon Green
    /// as the one small accent. Night is the dark ground.
    #[test]
    fn the_light_theme_is_the_brand_palette() {
        let t = tokens().theme(Mode::Light);
        assert_eq!(t.text, INK);
        assert_eq!(t.surface, PAPER);
        assert_eq!(t.text_dim, SLATE);
        assert_eq!(t.accent, MARGIN_BLUE);
        assert_eq!((t.brand, t.on_brand), (INK, PAPER));
        assert_eq!(t.highlight, RIBBON_GREEN);
        assert_eq!(tokens().theme(Mode::Dark).canvas, NIGHT);
        assert_eq!(tokens().theme(Mode::Dark).on_brand, PAPER);
    }

    /// WCAG AA in every theme: 4.5:1 for text (secondary text, links and problems included) on
    /// every surface it sits on, and 3:1 for the ribbon accent on the brand fill.
    #[test]
    fn text_and_focus_meet_wcag_aa() {
        for mode in MODES {
            let t = tokens().theme(mode);
            let grounds = [t.canvas, t.surface, t.surface_high, t.input, t.hover];
            for (name, fg) in [("text", t.text), ("text_dim", t.text_dim), ("accent", t.accent), ("danger", t.danger)] {
                for bg in grounds {
                    let r = contrast(fg, bg);
                    assert!(r >= 4.5, "{mode:?}: {name} {fg:?} on {bg:?} is {r:.2}:1");
                }
            }
            for (name, fg, bg, need) in [
                ("text on accent_soft", t.text, t.accent_soft, 4.5),
                ("accent on accent_soft", t.accent, t.accent_soft, 4.5),
                ("on_brand on brand", t.on_brand, t.brand, 4.5),
                ("highlight on brand", t.highlight, t.brand, 3.0),
            ] {
                let r = contrast(fg, bg);
                assert!(r >= need, "{mode:?}: {name} is {r:.2}:1");
            }
        }
    }

    #[test]
    fn the_icon_and_mask_are_pngs() {
        assert_eq!(&ICON_256[..8], b"\x89PNG\r\n\x1a\n");
        let mask = image::load_from_memory(MARK_MASK).expect("the mask decodes");
        // Drawn from the whole view box, two pixels a unit.
        assert_eq!((mask.width(), mask.height()), (626, 704));
        let ratio = mask.width() as f32 / mask.height() as f32;
        assert!((ratio - MARK_SIZE.0 / MARK_SIZE.1).abs() < 0.01);
    }

    /// The Windows program's icon (embedded by `apps/book/build.rs`) holds the sizes Windows
    /// asks for, each a 32-bit image.
    #[test]
    fn the_windows_icon_has_every_size() {
        let ico: &[u8] = include_bytes!("../../../assets/brand/icon.ico");
        let word = |at: usize| ico.get(at..at + 2).map_or(0, |b| u16::from_le_bytes([b[0], b[1]]));
        let long = |at: usize| ico.get(at..at + 4).map_or(0, |b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])) as usize;
        assert_eq!((word(0), word(2)), (0, 1), "an icon file");
        let mut sizes = Vec::new();
        for i in 0..usize::from(word(4)) {
            let e = 6 + 16 * i;
            let side = ico.get(e).map_or(0, |&b| if b == 0 { 256 } else { u32::from(b) });
            assert_eq!(word(e + 6), 32, "{side} px is 32-bit");
            assert!(long(e + 12) + long(e + 8) <= ico.len(), "{side} px lies inside the file");
            sizes.push(side);
        }
        assert_eq!(sizes, [16, 32, 48, 64, 128, 256]);
    }
}
