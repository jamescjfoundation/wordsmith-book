// SPDX-License-Identifier: MIT OR Apache-2.0
//! Wordsmith Book's brand (`assets/brand/`, from the Wordsmith brand kit): its design tokens, its
//! app icon and its mark. The brand lives here, never in the core; reading the files, the
//! window's icon and the tinted mask are the core's (`jcj_theme::brand`).
//!
//! The mark is the letter B followed by the ribbon cursor, as
//! `assets/brand/wordsmith-book-mark-ink.svg`. The B is drawn from a white mask
//! (`assets/brand/mark-mask.png`, made by `tools/brand-mask.py`) tinted at paint time, so one
//! picture serves every theme; the ribbon is plain shapes in Ribbon Green.

use jcj_shell::egui::{Color32, Painter, Rect, Shape, Stroke, pos2, vec2};
use jcj_shell::theme::{Brand, Mask};

/// The brand's files: its design tokens (`assets/brand/tokens.json`) and the app icon at 256
/// pixels, for the window. Reading them, the window's icon and installing the theme in a context
/// the shell does not manage (`BRAND.apply`, for tests and offscreen pictures) are the core's.
pub const BRAND: Brand = Brand::new(include_str!("../../../assets/brand/tokens.json"), include_bytes!("../../../assets/brand/icon-256.png"));

/// The B of the mark, as a mask tinted when it is painted.
const MARK: Mask = Mask::new("wordsmith-book-mark", include_bytes!("../../../assets/brand/mark-mask.png"));

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
    MARK.paint(painter, Rect::from_min_max(at(0.0, 0.0), at(MARK_SIZE.0, MARK_SIZE.1)), ink);
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
    use jcj_shell::theme::brand::{contrast, decode_png, ico_sizes};
    use jcj_shell::theme::{Mode, Tokens};

    #[test]
    fn the_shipped_tokens_read_and_differ_from_the_neutral_set() {
        let t = BRAND.read_tokens().expect("tokens.json reads");
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

    /// The light theme is the brand kit's palette: Iron Gall Ink on Page White, Slate for
    /// secondary text, Margin Blue for links and focus, ink for the brand fill, and Ribbon Green
    /// as the one small accent. Night is the dark ground.
    #[test]
    fn the_light_theme_is_the_brand_palette() {
        let t = BRAND.tokens().theme(Mode::Light);
        assert_eq!(t.text, INK);
        assert_eq!(t.surface, PAPER);
        assert_eq!(t.text_dim, SLATE);
        assert_eq!(t.accent, MARGIN_BLUE);
        assert_eq!((t.brand, t.on_brand), (INK, PAPER));
        assert_eq!(t.highlight, RIBBON_GREEN);
        assert_eq!(BRAND.tokens().theme(Mode::Dark).canvas, NIGHT);
        assert_eq!(BRAND.tokens().theme(Mode::Dark).on_brand, PAPER);
    }

    /// WCAG AA in every theme: 4.5:1 for text (secondary text, links and problems included) on
    /// every surface it sits on, and 3:1 for the ribbon accent on the brand fill.
    #[test]
    fn text_and_focus_meet_wcag_aa() {
        for mode in MODES {
            let t = BRAND.tokens().theme(mode);
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
        assert!(BRAND.icon().is_some(), "the window's icon decodes");
        let (width, height, _) = decode_png(MARK.png).expect("the mask decodes");
        // Drawn from the whole view box, two pixels a unit.
        assert_eq!((width, height), (626, 704));
        let ratio = width as f32 / height as f32;
        assert!((ratio - MARK_SIZE.0 / MARK_SIZE.1).abs() < 0.01);
    }

    /// The Windows program's icon (embedded by `apps/book/build.rs`) holds the sizes Windows
    /// asks for, each a 32-bit image.
    #[test]
    fn the_windows_icon_has_every_size() {
        let sizes = ico_sizes(include_bytes!("../../../assets/brand/icon.ico")).expect("a whole icon file of 32-bit pictures");
        assert_eq!(sizes, [16, 32, 48, 64, 128, 256]);
    }
}
