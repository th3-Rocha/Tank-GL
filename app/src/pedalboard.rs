//! The current preset drawn as the pedalboard: each pedal with its own
//! knobs and footswitch. Order of the Tank-G's panel, left to right:
//! IR, REVERB, DELAY, MOD, AMP and GATE at the end.

use egui::{
    Align2, Color32, Id, Painter, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, UiBuilder, pos2, vec2,
};
use tankg_proto::PATCH_LEN;

use crate::theme::*;
use crate::widgets::{self, Knob, bold, dim};

pub const AMP_NAMES: [&str; 9] = [
    "J120 CL",
    "J900 CL",
    "VAVE TS",
    "RAT OD",
    "VAVE HTS",
    "VAVE DS",
    "MS GV2",
    "J900 HV",
    "MS HIGAIN",
];
const IR_NAMES: [&str; 9] = ["OFF", "1", "2", "3", "4", "5", "6", "7", "8"];

/// Byte of each field in the 21-byte record (docs/PROTOCOL.md, section 0).
pub mod off {
    pub const AMP_ON: usize = 0;
    pub const MOD_ON: usize = 1;
    pub const DELAY_ON: usize = 2;
    pub const REVERB_ON: usize = 3;
    pub const GATE: usize = 4;
    pub const AMP_TYPE: usize = 5;
    pub const GAIN: usize = 6;
    pub const TREBLE: usize = 7;
    pub const MIDDLE: usize = 8;
    pub const BASS: usize = 9;
    pub const VOLUME: usize = 10;
    pub const MOD_FX: usize = 11;
    pub const MOD_SPEED: usize = 12;
    pub const DELAY_MIX: usize = 13;
    pub const DELAY_TIME: usize = 14;
    pub const REVERB_DECAY: usize = 15;
    pub const REVERB_MIX: usize = 16;
    pub const IR_CAB: usize = 17;
    /// Bytes 0–17 are the known fields.
    pub const KNOWN: usize = 18;
}

#[derive(Clone, Copy)]
enum Pedal {
    Cab,
    /// (name, color, on/off byte, left knob, right knob)
    Stomp(
        &'static str,
        Color32,
        usize,
        (usize, &'static str),
        (usize, &'static str),
    ),
    Amp,
    Gate,
}

const PEDALS: [Pedal; 6] = [
    Pedal::Cab,
    Pedal::Stomp(
        "REVERB",
        PURPLE,
        off::REVERB_ON,
        (off::REVERB_DECAY, "DECAY"),
        (off::REVERB_MIX, "MIX"),
    ),
    Pedal::Stomp(
        "DELAY",
        BLUE,
        off::DELAY_ON,
        (off::DELAY_MIX, "MIX"),
        (off::DELAY_TIME, "TIME"),
    ),
    Pedal::Stomp(
        "MOD",
        GREEN,
        off::MOD_ON,
        (off::MOD_FX, "FX"),
        (off::MOD_SPEED, "SPEED"),
    ),
    Pedal::Amp,
    Pedal::Gate,
];

const GAP: f32 = 26.0;
const MARGIN: f32 = 28.0;
/// Space above the tallest pedal (amp handle) and below the floor (names).
const TOP: f32 = 34.0;
const BOTTOM: f32 = 62.0;
const TALLEST: f32 = 320.0;

/// Width and height of each pedal, in points at scale 1.
fn size(p: Pedal) -> (f32, f32) {
    match p {
        Pedal::Cab => (270.0, 320.0),
        Pedal::Stomp(..) => (170.0, 300.0),
        Pedal::Amp => (440.0, 280.0),
        Pedal::Gate => (120.0, 230.0),
    }
}

/// Draws the pedals and returns the edits (byte, new value).
/// `max_h` = maximum height, so the screen fits without scrolling.
pub fn show(
    ui: &mut Ui,
    v: &[u8; PATCH_LEN],
    saved: Option<&[u8; PATCH_LEN]>,
    online: bool,
    max_h: f32,
) -> Vec<(usize, u8)> {
    const ROW_H: f32 = TOP + TALLEST + BOTTOM;
    let natural = |row: &[Pedal]| {
        row.iter().map(|p| size(*p).0).sum::<f32>() + GAP * (row.len() - 1) as f32 + MARGIN * 2.0
    };
    let avail = ui.available_width();
    // One row, or two (stomps on top, amp and gate below) when the window is
    // narrow and tall enough that two rows draw the pedals noticeably bigger.
    let one: [&[Pedal]; 1] = [&PEDALS];
    let two: [&[Pedal]; 2] = [&PEDALS[..4], &PEDALS[4..]];
    let scale = |rows: &[&[Pedal]]| {
        let widest = rows.iter().map(|r| natural(r)).fold(0.0, f32::max);
        (avail / widest)
            .min(max_h / (ROW_H * rows.len() as f32))
            .clamp(0.45, 1.6)
    };
    let rows: &[&[Pedal]] = if scale(&two) > scale(&one) * 1.15 {
        &two
    } else {
        &one
    };
    let s = scale(rows);
    let (rect, _) =
        ui.allocate_exact_size(vec2(avail, ROW_H * s * rows.len() as f32), Sense::hover());
    ui.painter().rect_filled(rect, 12, SURFACE);

    let mut b = Board {
        ui,
        v,
        saved,
        s,
        online,
        edits: Vec::new(),
    };
    for (i, row) in rows.iter().enumerate() {
        let floor = rect.top() + (ROW_H * i as f32 + TOP + TALLEST) * s;
        b.ui.painter().line_segment(
            [
                pos2(rect.left() + 16.0, floor),
                pos2(rect.right() - 16.0, floor),
            ],
            Stroke::new(1.0, LINE),
        );
        let x = rect.left() + (avail - natural(row) * s) / 2.0 + MARGIN * s;
        b.row(row, x, floor);
    }
    b.edits
}

impl Board<'_> {
    fn row(&mut self, pedals: &[Pedal], mut x: f32, floor: f32) {
        let (v, s) = (self.v, self.s);
        let b = self;
        for &p in pedals {
            let (w, h) = size(p);
            let r = Rect::from_min_size(pos2(x, floor - h * s), vec2(w * s, h * s));
            let (name, on, sub) = match p {
                Pedal::Cab => (
                    "CAB · IR",
                    v[off::IR_CAB] > 0,
                    if v[off::IR_CAB] > 0 {
                        format!("Slot {}", v[off::IR_CAB])
                    } else {
                        "OFF".into()
                    },
                ),
                Pedal::Stomp(name, _, t, _, _) => {
                    let sub = match t {
                        off::REVERB_ON => format!("Decay {}", v[off::REVERB_DECAY]),
                        off::DELAY_ON => format!("Time {}", v[off::DELAY_TIME]),
                        _ => format!("FX {}", v[off::MOD_FX]),
                    };
                    (name, v[t] != 0, sub)
                }
                Pedal::Amp => (
                    "AMP",
                    v[off::AMP_ON] != 0,
                    amp_name(v[off::AMP_TYPE]).to_owned(),
                ),
                Pedal::Gate => (
                    "GATE",
                    v[off::GATE] > 0,
                    if v[off::GATE] > 0 {
                        format!("Gate {}", v[off::GATE])
                    } else {
                        "Open".into()
                    },
                ),
            };
            match p {
                Pedal::Cab => b.cab(r),
                Pedal::Stomp(name, color, t, k1, k2) => b.stomp(r, name, color, t, k1, k2),
                Pedal::Amp => b.amp(r),
                Pedal::Gate => b.gate(r),
            }
            let color = color(p);
            let radius = 14.0 * s;
            let p = b.ui.painter();
            if on {
                // On: outline and glow in the pedal's color.
                for i in 1..=4 {
                    let a = 0.22 / i as f32;
                    p.rect_stroke(
                        r.expand(i as f32 * 3.0 * s),
                        radius + i as f32 * 3.0 * s,
                        Stroke::new(3.0 * s, color.gamma_multiply(a)),
                        StrokeKind::Outside,
                    );
                }
                p.rect_stroke(r, radius, Stroke::new(2.0 * s, color), StrokeKind::Inside);
            } else {
                // Off: the whole pedal, knobs included, is dimmed.
                p.rect_filled(r, radius, Color32::from_black_alpha(150));
            }
            let (lbl, lc) = if on {
                (format!("● {name}"), color)
            } else {
                (format!("{name} · OFF"), FAINT)
            };
            p.text(
                pos2(r.center().x, floor + 12.0 * s),
                Align2::CENTER_TOP,
                lbl,
                bold(17.0 * s),
                lc,
            );
            p.text(
                pos2(r.center().x, floor + 36.0 * s),
                Align2::CENTER_TOP,
                sub,
                egui::FontId::monospace(13.0 * s),
                MUTED,
            );
            x += (w + GAP) * s;
        }
    }
}

fn color(p: Pedal) -> Color32 {
    match p {
        Pedal::Cab => YELLOW,
        Pedal::Stomp(_, c, ..) => c,
        Pedal::Amp => RED,
        Pedal::Gate => ACCENT,
    }
}

fn amp_name(t: u8) -> &'static str {
    AMP_NAMES.get(t as usize).copied().unwrap_or("?")
}

struct Board<'a> {
    ui: &'a mut Ui,
    v: &'a [u8; PATCH_LEN],
    saved: Option<&'a [u8; PATCH_LEN]>,
    s: f32,
    online: bool,
    edits: Vec<(usize, u8)>,
}

impl Board<'_> {
    fn at(&self, r: Rect, x: f32, y: f32) -> Pos2 {
        pos2(r.left() + x * self.s, r.top() + y * self.s)
    }

    fn painter(&self) -> Painter {
        self.ui.painter().clone()
    }

    /// Knob centered on `cx`, label top at `y` (pedal points).
    #[allow(clippy::too_many_arguments)]
    fn knob(
        &mut self,
        r: Rect,
        cx: f32,
        y: f32,
        d: f32,
        o: usize,
        label: &str,
        max: u8,
        color: Color32,
    ) {
        let d = d * self.s;
        let size = widgets::knob_size(d);
        let top = self.at(r, cx, y);
        let kr = Rect::from_min_size(pos2(top.x - size.x / 2.0, top.y), size);
        let names: Option<&[&str]> = match o {
            off::AMP_TYPE => Some(&AMP_NAMES),
            off::IR_CAB => Some(&IR_NAMES),
            _ => None,
        };
        let k = Knob {
            label,
            value: self.v[o].min(max),
            max,
            diameter: d,
            color,
            names,
            saved: self.saved.map(|s| s[o]),
        };
        let mut child = self.ui.new_child(UiBuilder::new().max_rect(kr));
        if !self.online {
            child.disable();
        }
        if let Some(nv) = widgets::knob(&mut child, Id::new(("knob", o)), k) {
            self.edits.push((o, nv));
        }
    }

    /// Round button that toggles byte `o`.
    fn footswitch(&mut self, c: Pos2, radius: f32, o: usize, name: &str) {
        let on = self.v[o] != 0;
        let r = Rect::from_center_size(c, egui::Vec2::splat(radius * 2.0 + 6.0));
        let resp = self
            .ui
            .interact(
                r,
                Id::new(("fsw", o)),
                if self.online {
                    Sense::click()
                } else {
                    Sense::hover()
                },
            )
            .on_hover_text(format!("Turn {name} {}", if on { "off" } else { "on" }));
        let p = self.painter();
        p.circle_filled(c, radius + 4.0, Color32::from_gray(12));
        p.circle_stroke(
            c,
            radius + 4.0,
            Stroke::new(
                1.5,
                if resp.hovered() && self.online {
                    ACCENT
                } else {
                    Color32::from_gray(74)
                },
            ),
        );
        p.circle_filled(c, radius, METAL);
        p.circle_filled(c, radius * 0.72, Color32::from_gray(160));
        p.circle_filled(
            c - vec2(radius * 0.3, radius * 0.35),
            radius * 0.28,
            Color32::from_white_alpha(110),
        );
        if resp.clicked() {
            self.edits.push((o, (!on) as u8));
        }
    }

    fn led(&self, c: Pos2, r: f32, color: Color32, on: bool) {
        let p = self.painter();
        if on {
            p.circle_filled(c, r * 2.4, color.gamma_multiply(0.22));
        }
        p.circle_filled(c, r, if on { color } else { Color32::from_gray(26) });
        p.circle_stroke(c, r, Stroke::new(1.0, Color32::BLACK));
    }

    /// Colored stripe at the top of `r`. It is the rounded rectangle itself,
    /// clipped at the stripe's height, so the corners follow the pedal's curve
    /// (a rectangle shorter than the radius would get smaller corners and leak).
    fn stripe(&self, r: Rect, radius: f32, height: f32, color: Color32) {
        let band = Rect::from_min_size(r.min, vec2(r.width(), height * self.s));
        self.painter()
            .with_clip_rect(band.intersect(self.ui.clip_rect()))
            .rect_filled(r, radius * self.s, color);
    }

    fn body(&self, r: Rect, radius: f32) {
        let p = self.painter();
        let rr = radius * self.s;
        p.rect(
            r,
            rr,
            Color32::from_gray(40),
            Stroke::new(1.0, Color32::BLACK),
            StrokeKind::Inside,
        );
        // highlight on top, like brushed metal
        let top = Rect::from_min_max(r.min, pos2(r.right(), r.center().y));
        p.rect_filled(top.shrink(1.0), rr, Color32::from_white_alpha(5));
    }

    fn stomp(
        &mut self,
        r: Rect,
        name: &str,
        color: Color32,
        t: usize,
        k1: (usize, &str),
        k2: (usize, &str),
    ) {
        let on = self.v[t] != 0;
        let s = self.s;
        self.body(r, 14.0);
        let p = self.painter();
        self.stripe(r, 14.0, 10.0, dim(color, on));
        let w = r.width() / s;
        self.knob(r, w * 0.27, 22.0, 46.0, k1.0, k1.1, 127, dim(color, on));
        self.knob(r, w * 0.73, 22.0, 46.0, k2.0, k2.1, 127, dim(color, on));
        self.led(self.at(r, w / 2.0, 140.0), 7.0 * s, color, on);
        p.text(
            self.at(r, w / 2.0, 172.0),
            Align2::CENTER_CENTER,
            name,
            bold(22.0 * s),
            if on { INK } else { MUTED },
        );
        self.footswitch(self.at(r, w / 2.0, 245.0), 26.0 * s, t, name);
    }

    fn gate(&mut self, r: Rect) {
        let on = self.v[off::GATE] > 0;
        let s = self.s;
        self.body(r, 12.0);
        let p = self.painter();
        self.stripe(r, 12.0, 8.0, dim(ACCENT, on));
        let w = r.width() / s;
        self.knob(
            r,
            w / 2.0,
            20.0,
            50.0,
            off::GATE,
            "GATE",
            127,
            dim(ACCENT, on),
        );
        self.led(self.at(r, w / 2.0, 140.0), 7.0 * s, ACCENT, on);
        p.text(
            self.at(r, w / 2.0, 180.0),
            Align2::CENTER_CENTER,
            "GATE",
            bold(20.0 * s),
            if on { INK } else { MUTED },
        );
    }

    fn amp(&mut self, r: Rect) {
        let on = self.v[off::AMP_ON] != 0;
        let s = self.s;
        let w = r.width() / s;
        let h = r.height() / s;
        let p = self.painter();
        // handle
        let handle =
            Rect::from_min_size(self.at(r, w / 2.0 - 40.0, -18.0), vec2(80.0 * s, 22.0 * s));
        p.rect_stroke(
            handle,
            10.0 * s,
            Stroke::new(4.0 * s, METAL),
            StrokeKind::Middle,
        );
        p.rect(
            r,
            14.0 * s,
            TOLEX,
            Stroke::new(1.5, Color32::from_gray(44)),
            StrokeKind::Inside,
        );
        // knob panel
        let panel = Rect::from_min_size(self.at(r, 12.0, 12.0), vec2((w - 24.0) * s, 128.0 * s));
        p.rect(
            panel,
            6.0 * s,
            Color32::from_gray(52),
            Stroke::new(1.0, METAL),
            StrokeKind::Inside,
        );
        self.stripe(panel, 6.0, 3.0, dim(RED, on));
        let knobs = [
            (off::AMP_TYPE, "TYPE", 8u8, 42.0),
            (off::GAIN, "GAIN", 127, 46.0),
            (off::TREBLE, "TREBLE", 127, 40.0),
            (off::MIDDLE, "MIDDLE", 127, 40.0),
            (off::BASS, "BASS", 127, 40.0),
            (off::VOLUME, "VOLUME", 127, 46.0),
        ];
        let slot = (w - 24.0) / knobs.len() as f32;
        for (i, (o, label, max, d)) in knobs.into_iter().enumerate() {
            self.knob(
                r,
                12.0 + slot * (i as f32 + 0.5),
                20.0,
                d,
                o,
                label,
                max,
                dim(RED, on),
            );
        }
        // grille with the name plate
        let grille = Rect::from_min_max(self.at(r, 12.0, 150.0), self.at(r, w - 12.0, h - 12.0));
        widgets::grille(&p, grille, s);
        let plate = Rect::from_center_size(grille.center(), vec2(170.0 * s, 40.0 * s));
        p.rect(
            plate,
            4.0 * s,
            Color32::from_gray(17),
            Stroke::new(1.0, METAL),
            StrokeKind::Inside,
        );
        p.text(
            plate.center(),
            Align2::CENTER_CENTER,
            amp_name(self.v[off::AMP_TYPE]),
            bold(20.0 * s),
            if on { RED } else { MUTED },
        );
        // power light
        let jewel = pos2(plate.right() + 40.0 * s, grille.center().y);
        let jr = Rect::from_center_size(jewel, egui::Vec2::splat(34.0 * s));
        let resp = self
            .ui
            .interact(
                jr,
                Id::new("amp-power"),
                if self.online {
                    Sense::click()
                } else {
                    Sense::hover()
                },
            )
            .on_hover_text(if on { "Turn AMP off" } else { "Turn AMP on" });
        p.circle_filled(jewel, 15.0 * s, Color32::from_gray(12));
        p.circle_stroke(
            jewel,
            15.0 * s,
            Stroke::new(
                1.5,
                if resp.hovered() && self.online {
                    ACCENT
                } else {
                    METAL
                },
            ),
        );
        self.led(jewel, 9.0 * s, RED, on);
        if resp.clicked() {
            self.edits.push((off::AMP_ON, (!on) as u8));
        }
    }

    fn cab(&mut self, r: Rect) {
        let ir = self.v[off::IR_CAB];
        let on = ir > 0;
        let s = self.s;
        let w = r.width() / s;
        let h = r.height() / s;
        let p = self.painter();
        p.rect(
            r,
            14.0 * s,
            TOLEX,
            Stroke::new(1.5, Color32::from_gray(44)),
            StrokeKind::Inside,
        );
        let grille = Rect::from_min_max(self.at(r, 12.0, 12.0), self.at(r, w - 12.0, 200.0));
        widgets::grille(&p, grille, s);
        for fx in [0.27, 0.73] {
            let c = self.at(r, w * fx, 106.0);
            p.circle_filled(c, 54.0 * s, Color32::from_gray(10));
            p.circle_filled(c, 48.0 * s, Color32::from_gray(30));
            p.circle_stroke(c, 34.0 * s, Stroke::new(1.0, Color32::from_white_alpha(14)));
            p.circle_stroke(c, 20.0 * s, Stroke::new(1.0, Color32::from_white_alpha(14)));
            p.circle_filled(c, 15.0 * s, Color32::from_gray(70));
            if on {
                p.circle_stroke(
                    c,
                    51.0 * s,
                    Stroke::new(2.0 * s, YELLOW.gamma_multiply(0.75)),
                );
            }
        }
        // lower panel: IR knob and the slot plate
        let panel = Rect::from_min_max(self.at(r, 12.0, 210.0), self.at(r, w - 12.0, h - 12.0));
        p.rect(
            panel,
            6.0 * s,
            Color32::from_gray(52),
            Stroke::new(1.0, METAL),
            StrokeKind::Inside,
        );
        self.knob(
            r,
            62.0,
            214.0,
            40.0,
            off::IR_CAB,
            "IR CAB",
            8,
            dim(YELLOW, on),
        );
        let plate = Rect::from_min_max(self.at(r, 120.0, 236.0), self.at(r, w - 24.0, 278.0));
        p.rect_filled(
            plate,
            4.0 * s,
            if on {
                YELLOW.gamma_multiply(0.9)
            } else {
                Color32::from_gray(110)
            },
        );
        let t = if on {
            format!("IR · SLOT {ir}")
        } else {
            "IR · OFF".to_owned()
        };
        p.text(
            plate.center(),
            Align2::CENTER_CENTER,
            t,
            bold(16.0 * s),
            Color32::from_gray(17),
        );
    }
}
