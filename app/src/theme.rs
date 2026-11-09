//! Tank Studio colors (design/tokens.json) and the egui theme.

use egui::{Color32, CornerRadius, Stroke, Visuals};

const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

pub const GROUND: Color32 = hex(0x071a1c);
pub const SURFACE: Color32 = hex(0x0b2124);
pub const CARD: Color32 = hex(0x0f292c);
pub const RAISED: Color32 = hex(0x143236);
pub const CONTROL: Color32 = hex(0x1d4044);
pub const CONTROL_HOVER: Color32 = hex(0x265157);
pub const CONTROL_ACTIVE: Color32 = hex(0x173539);
pub const LINE: Color32 = hex(0x1d3f43);
pub const LINE_STRONG: Color32 = hex(0x4f7f84);
pub const INK: Color32 = Color32::WHITE;
pub const MUTED: Color32 = hex(0x9db7ba);
pub const FAINT: Color32 = hex(0x5e7f83);

/// Accent: the cyan of the Tank-G's body.
pub const ACCENT: Color32 = hex(0x22d3ee);
pub const ACCENT_SOFT: Color32 = hex(0x06323a);
pub const ON_ACCENT: Color32 = hex(0x001a1f);

pub const OK: Color32 = hex(0x3fd0c9);
pub const DANGER: Color32 = hex(0xff6b5e);
pub const DANGER_SOFT: Color32 = hex(0x3d1714);
pub const WARN: Color32 = hex(0xffd54a);
pub const WARN_SOFT: Color32 = hex(0x3a3110);

pub const DISPLAY_BG: Color32 = hex(0x070708);
pub const DISPLAY_ON: Color32 = hex(0xff4d2e);
pub const DISPLAY_OFF: Color32 = hex(0x2a1410);

pub const TOLEX: Color32 = hex(0x0c0b0d);
pub const GRILLE: Color32 = hex(0x2a2622);
pub const METAL: Color32 = hex(0x8b8a8f);

/// Each block's color, as on the pedal.
pub const RED: Color32 = hex(0xff3b30);
pub const YELLOW: Color32 = hex(0xffd400);
pub const GREEN: Color32 = hex(0x34e05a);
pub const BLUE: Color32 = hex(0x3b82f6);
pub const PURPLE: Color32 = hex(0xa855f7);

pub fn visuals() -> Visuals {
    let mut v = Visuals::dark();
    let r = CornerRadius::same(6);
    v.panel_fill = GROUND;
    v.window_fill = CARD;
    v.extreme_bg_color = DISPLAY_BG;
    v.faint_bg_color = RAISED;
    v.hyperlink_color = ACCENT;
    v.selection.bg_fill = ACCENT_SOFT;
    v.selection.stroke = Stroke::new(1.0, ACCENT);
    v.window_stroke = Stroke::new(1.0, LINE);
    v.override_text_color = None;

    let w = &mut v.widgets;
    w.noninteractive.bg_fill = CARD;
    w.noninteractive.weak_bg_fill = CARD;
    w.noninteractive.bg_stroke = Stroke::new(1.0, LINE);
    w.noninteractive.fg_stroke = Stroke::new(1.0, MUTED);
    w.inactive.bg_fill = CONTROL;
    w.inactive.weak_bg_fill = CONTROL;
    w.inactive.fg_stroke = Stroke::new(1.0, INK);
    w.hovered.bg_fill = CONTROL_HOVER;
    w.hovered.weak_bg_fill = CONTROL_HOVER;
    w.hovered.bg_stroke = Stroke::new(1.0, LINE_STRONG);
    w.hovered.fg_stroke = Stroke::new(1.0, INK);
    w.active.bg_fill = CONTROL_ACTIVE;
    w.active.weak_bg_fill = CONTROL_ACTIVE;
    w.active.bg_stroke = Stroke::new(1.0, ACCENT);
    w.active.fg_stroke = Stroke::new(1.0, INK);
    for s in [
        &mut w.noninteractive,
        &mut w.inactive,
        &mut w.hovered,
        &mut w.active,
        &mut w.open,
    ] {
        s.corner_radius = r;
    }
    v
}
