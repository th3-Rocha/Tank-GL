//! Hand-drawn parts: knob, LED display, footswitch and the signal chain.

use std::f32::consts::PI;

use egui::{
    Align2, Color32, CornerRadius, FontId, Id, Key, Painter, Pos2, Rect, Response, Sense, Stroke,
    StrokeKind, Ui, Vec2, pos2, vec2,
};
use fastframe_fonts::Weight;

use crate::theme::*;

pub fn bold(size: f32) -> FontId {
    Weight::Bold.font_id(size)
}

pub fn semibold(size: f32) -> FontId {
    Weight::SemiBold.font_id(size)
}

/// Darkens a color for blocks that are off.
pub fn dim(c: Color32, on: bool) -> Color32 {
    if on { c } else { c.gamma_multiply(0.45) }
}

/// Card frame with a title; `right` draws to the right of the title.
pub fn card<R>(
    ui: &mut Ui,
    title: &str,
    right: impl FnOnce(&mut Ui),
    body: impl FnOnce(&mut Ui) -> R,
) -> R {
    egui::Frame::new()
        .fill(CARD)
        .stroke(Stroke::new(1.0, LINE))
        .corner_radius(10)
        .inner_margin(14)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(title).font(semibold(15.0)).color(INK));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), right);
            });
            ui.add_space(10.0);
            body(ui)
        })
        .inner
}

/// Colored tag (ok, warning, error).
pub fn tag(ui: &mut Ui, text: &str, fg: Color32, bg: Color32) -> Response {
    egui::Frame::new()
        .fill(bg)
        .corner_radius(CornerRadius::same(99))
        .inner_margin(vec2(8.0, 2.0))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).font(semibold(12.0)).color(fg))
        })
        .response
}

/// Accent button (cyan background).
pub fn primary_button(ui: &mut Ui, text: &str, enabled: bool) -> Response {
    let b = egui::Button::new(
        egui::RichText::new(text)
            .font(semibold(13.0))
            .color(if enabled { ON_ACCENT } else { FAINT }),
    )
    .fill(if enabled { ACCENT } else { CONTROL })
    .min_size(vec2(0.0, 30.0));
    ui.add_enabled(enabled, b)
}

// ---------------------------------------------------------------- knob

pub struct Knob<'a> {
    pub label: &'a str,
    pub value: u8,
    pub max: u8,
    pub diameter: f32,
    pub color: Color32,
    /// Name of each value (amp type, IR slot).
    pub names: Option<&'a [&'a str]>,
    /// Value saved in flash: double-click goes back to it.
    pub saved: Option<u8>,
}

/// Draws the knob and returns the new value when it changes.
/// Vertical drag, mouse wheel or arrow keys; Shift = fine adjust.
pub fn knob(ui: &mut Ui, id: Id, k: Knob<'_>) -> Option<u8> {
    let d = k.diameter;
    let size = knob_size(d);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let resp = resp.on_hover_text(match k.saved {
        Some(s) => format!(
            "Drag, scroll or use the arrow keys · Shift = fine · double-click = saved ({s})"
        ),
        None => "Drag, scroll or use the arrow keys · Shift = fine".to_owned(),
    });
    let max = k.max as f32;
    let fine = ui.input(|i| i.modifiers.shift);
    let mut new = None;

    if resp.clicked() || resp.drag_started() {
        resp.request_focus();
    }
    if resp.drag_started() {
        ui.data_mut(|m| m.insert_temp(id, k.value as f32));
    }
    if resp.dragged() {
        // A 0–127 knob needs ~200 px to go end to end.
        let px = if fine { 800.0 } else { 200.0 };
        let mut acc = ui.data(|m| m.get_temp::<f32>(id)).unwrap_or(k.value as f32);
        acc = (acc - resp.drag_delta().y * max / px).clamp(0.0, max);
        ui.data_mut(|m| m.insert_temp(id, acc));
        new = Some(acc.round() as u8);
    }
    if resp.hovered() && resp.enabled() {
        // Scrolling over the knob belongs to it, not the page. egui spreads each
        // wheel turn over several frames, so it is cleared every frame.
        ui.input_mut(|i| i.smooth_scroll_delta = Vec2::ZERO);
        // Mouse wheel: each "notch" is one step; a touchpad accumulates 40 px per step.
        let wheel: f32 = ui.input(|i| {
            i.events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::MouseWheel { unit, delta, .. } => {
                        Some(if *unit == egui::MouseWheelUnit::Point {
                            delta.y / 40.0
                        } else {
                            delta.y
                        })
                    }
                    _ => None,
                })
                .sum()
        });
        if wheel != 0.0 {
            let wid = id.with("wheel");
            let acc = ui.data(|m| m.get_temp::<f32>(wid)).unwrap_or(0.0) + wheel;
            let steps = acc.trunc();
            ui.data_mut(|m| m.insert_temp(wid, acc - steps));
            if steps != 0.0 {
                let step = if fine || k.max <= 8 { 1 } else { 3 };
                let delta = steps as i32 * step;
                new = Some((k.value as i32 + delta).clamp(0, k.max as i32) as u8);
            }
        }
    }
    if resp.has_focus() {
        let (up, down) = ui.input(|i| {
            (
                i.key_pressed(Key::ArrowUp) || i.key_pressed(Key::ArrowRight),
                i.key_pressed(Key::ArrowDown) || i.key_pressed(Key::ArrowLeft),
            )
        });
        let step = if fine || k.max <= 8 { 1 } else { 5 };
        if up {
            new = Some(k.value.saturating_add(step).min(k.max));
        } else if down {
            new = Some(k.value.saturating_sub(step));
        }
    }
    if resp.double_clicked() {
        new = k.saved;
    }

    let shown = new.unwrap_or(k.value);
    let p = ui.painter();
    p.text(
        pos2(rect.center().x, rect.top()),
        Align2::CENTER_TOP,
        k.label,
        semibold(label_size(d)),
        MUTED,
    );
    let c = pos2(
        rect.center().x,
        rect.top() + label_size(d) + LABEL_GAP + d / 2.0,
    );
    let r = d / 2.0;
    let t = if k.max == 0 { 0.0 } else { shown as f32 / max };
    arc(p, c, r + 5.0, 0.0, 1.0, Stroke::new(3.0, LINE));
    arc(p, c, r + 5.0, 0.0, t, Stroke::new(3.0, k.color));
    p.circle_filled(c, r, Color32::from_gray(30));
    p.circle_filled(c, r * 0.86, Color32::from_gray(48));
    p.circle_stroke(c, r, Stroke::new(1.0, Color32::from_gray(80)));
    let a = knob_angle(t);
    p.line_segment(
        [
            c + vec2(a.cos(), a.sin()) * r * 0.25,
            c + vec2(a.cos(), a.sin()) * r * 0.78,
        ],
        Stroke::new(2.5, INK),
    );
    if resp.has_focus() {
        p.circle_stroke(c, r + 9.0, Stroke::new(1.0, ACCENT));
    }
    let text = match k.names {
        Some(n) => n.get(shown as usize).copied().unwrap_or("?").to_owned(),
        None => shown.to_string(),
    };
    let changed = k.saved.is_some_and(|s| s != shown);
    p.text(
        pos2(rect.center().x, rect.bottom()),
        Align2::CENTER_BOTTOM,
        text,
        FontId::monospace(value_size(d)),
        if changed { WARN } else { INK },
    );
    new.filter(|v| *v != k.value)
}

/// Space between the label and the top of the knob: the arc sits 5 px outside the knob.
const LABEL_GAP: f32 = 14.0;

fn label_size(d: f32) -> f32 {
    (d * 0.26).clamp(10.0, 14.0)
}

fn value_size(d: f32) -> f32 {
    (d * 0.26).clamp(11.0, 14.0)
}

/// Size [`knob`] takes with diameter `d`.
pub fn knob_size(d: f32) -> Vec2 {
    vec2(
        d + 30.0,
        label_size(d) + LABEL_GAP + d + 10.0 + value_size(d),
    )
}

/// 0 = seven thirty (bottom left), 1 = four thirty.
fn knob_angle(t: f32) -> f32 {
    0.75 * PI + t * 1.5 * PI
}

fn arc(p: &Painter, c: Pos2, r: f32, t0: f32, t1: f32, stroke: Stroke) {
    if t1 <= t0 {
        return;
    }
    let n = ((t1 - t0) * 48.0).ceil().max(2.0) as usize;
    let pts: Vec<Pos2> = (0..=n)
        .map(|i| {
            let a = knob_angle(t0 + (t1 - t0) * i as f32 / n as f32);
            c + vec2(a.cos(), a.sin()) * r
        })
        .collect();
    p.add(egui::Shape::line(pts, stroke));
}

// ---------------------------------------------------------------- display

/// One 7-segment digit, like the pedal's display. `None` = dash.
pub fn led_digit(ui: &mut Ui, digit: Option<u8>, height: f32) {
    let w = height * 0.62;
    let (rect, _) = ui.allocate_exact_size(vec2(w + 24.0, height + 24.0), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, 8, DISPLAY_BG);
    p.rect_stroke(rect, 8, Stroke::new(1.0, LINE), StrokeKind::Inside);
    let r = rect.shrink(12.0);
    let t = height * 0.11;
    // a b c d e f g
    const SEGS: [u8; 11] = [
        0x3f, 0x06, 0x5b, 0x4f, 0x66, 0x6d, 0x7d, 0x07, 0x7f, 0x6f, 0x40,
    ];
    let mask = SEGS[digit.map_or(10, |d| d.min(9) as usize)];
    let (l, rr, top, mid, bot) = (r.left(), r.right(), r.top(), r.center().y, r.bottom());
    let segs = [
        Rect::from_min_max(pos2(l + t, top), pos2(rr - t, top + t)),
        Rect::from_min_max(pos2(rr - t, top + t * 0.6), pos2(rr, mid - t * 0.4)),
        Rect::from_min_max(pos2(rr - t, mid + t * 0.4), pos2(rr, bot - t * 0.6)),
        Rect::from_min_max(pos2(l + t, bot - t), pos2(rr - t, bot)),
        Rect::from_min_max(pos2(l, mid + t * 0.4), pos2(l + t, bot - t * 0.6)),
        Rect::from_min_max(pos2(l, top + t * 0.6), pos2(l + t, mid - t * 0.4)),
        Rect::from_min_max(pos2(l + t, mid - t / 2.0), pos2(rr - t, mid + t / 2.0)),
    ];
    for (i, s) in segs.iter().enumerate() {
        let on = mask & (1 << i) != 0;
        p.rect_filled(*s, 2, if on { DISPLAY_ON } else { DISPLAY_OFF });
    }
}

// ---------------------------------------------------------------- footswitch

/// Big footswitch with LED. Returns the click.
pub fn footswitch(
    ui: &mut Ui,
    letter: &str,
    caption: &str,
    on: bool,
    led: Color32,
    enabled: bool,
) -> Response {
    let (rect, resp) = ui.allocate_exact_size(
        vec2(84.0, 112.0),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let p = ui.painter();
    let hovered = resp.hovered() && enabled;
    p.rect_filled(rect, 10, if hovered { RAISED } else { SURFACE });
    p.rect_stroke(
        rect,
        10,
        Stroke::new(1.0, if on { led.gamma_multiply(0.6) } else { LINE }),
        StrokeKind::Inside,
    );
    let lc = pos2(rect.center().x, rect.top() + 16.0);
    if on {
        p.circle_filled(lc, 10.0, led.gamma_multiply(0.25));
    }
    p.circle_filled(lc, 5.0, if on { led } else { Color32::from_gray(26) });
    let fc = pos2(rect.center().x, rect.top() + 54.0);
    p.circle_filled(fc, 24.0, Color32::from_gray(12));
    p.circle_stroke(
        fc,
        24.0,
        Stroke::new(
            1.0,
            if hovered {
                ACCENT
            } else {
                Color32::from_gray(74)
            },
        ),
    );
    p.circle_filled(fc, 18.0, METAL);
    p.circle_filled(fc - vec2(5.0, 6.0), 6.0, Color32::from_white_alpha(120));
    p.text(
        pos2(rect.center().x, rect.bottom() - 30.0),
        Align2::CENTER_CENTER,
        letter,
        bold(16.0),
        if enabled { INK } else { FAINT },
    );
    p.text(
        pos2(rect.center().x, rect.bottom() - 12.0),
        Align2::CENTER_CENTER,
        caption,
        FontId::monospace(11.0),
        MUTED,
    );
    resp
}

/// Speaker grille (diagonal weave).
pub fn grille(p: &Painter, r: Rect, s: f32) {
    p.rect_filled(r, 4.0 * s, hex_dark());
    let clip = p.with_clip_rect(r);
    let step = 6.0 * s;
    let mut x = r.left() - r.height();
    while x < r.right() {
        clip.line_segment(
            [pos2(x, r.bottom()), pos2(x + r.height(), r.top())],
            Stroke::new(2.0 * s, GRILLE),
        );
        x += step;
    }
    p.rect_stroke(
        r,
        4.0 * s,
        Stroke::new(1.0, Color32::BLACK),
        StrokeKind::Inside,
    );
}

fn hex_dark() -> Color32 {
    Color32::from_rgb(0x1b, 0x18, 0x15)
}

// ---------------------------------------------------------------- switch

/// On/off switch. `color` is the color when on.
pub fn switch(ui: &mut Ui, on: bool, color: Color32, enabled: bool) -> Response {
    let (rect, resp) = ui.allocate_exact_size(
        vec2(36.0, 20.0),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let t = ui.ctx().animate_bool_responsive(resp.id, on);
    let p = ui.painter();
    let track = if on {
        color.gamma_multiply(0.85)
    } else {
        Color32::from_gray(90)
    };
    p.rect_filled(
        rect,
        10,
        if enabled {
            track
        } else {
            track.gamma_multiply(0.4)
        },
    );
    let x = egui::lerp(rect.left() + 10.0..=rect.right() - 10.0, t);
    p.circle_filled(pos2(x, rect.center().y), 7.5, Color32::WHITE);
    if resp.hovered() && enabled {
        p.rect_stroke(rect, 10, Stroke::new(1.0, INK), StrokeKind::Outside);
    }
    resp
}
