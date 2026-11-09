//! Tank Studio's screens.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use egui::{Color32, Key, RichText, Stroke, Ui, vec2};
use tankg_proto::{Mode, PATCH_LEN, Patch, PresetId, Status, preset};

use crate::device::{Cmd, Device, Event, Tone};
use crate::pedalboard::{self, off};
use crate::theme::*;
use crate::widgets::{self, bold, card, semibold};

/// How long the on-screen value wins over the one read from the pedal after
/// an edit. Without it, the 600 ms poll makes the knob jump mid-drag.
const HOLD: Duration = Duration::from_millis(1200);
const LOG_MAX: usize = 200;
const LETTERS: [&str; 4] = ["A", "B", "C", "D"];
/// Space between cards and at the edges, the same everywhere.
const GAP: f32 = 12.0;
/// Content height of the two lower cards (equal).
const BOTTOM_ROW_H: f32 = 206.0;
/// Below this width the lower cards stack (vertical layout).
const VERTICAL_BELOW: f32 = 1250.0;
/// Everything but the pedals: card headers and margins, hint and spacing.
const RESERVED_H: f32 = 176.0;

enum Conn {
    Connecting,
    Connected(String),
    Disconnected(String),
}

struct LogLine {
    at: Duration,
    out: bool,
    bytes: String,
    text: String,
}

pub struct TankApp {
    dev: Device,
    conn: Conn,
    status: Option<Status>,
    /// Current preset as the pedal sent it.
    patch: Option<[u8; PATCH_LEN]>,
    /// The 36 presets in flash, to tell what was modified.
    bank: Option<Vec<Patch>>,
    loopback: Option<bool>,
    held: HashMap<usize, (u8, Instant)>,
    log: VecDeque<LogLine>,
    paused: bool,
    /// The MIDI monitor starts hidden.
    show_monitor: bool,
    /// Last preset seen. In bypass the pedal sends letter `ff`.
    last_preset: Option<PresetId>,
    /// "Save preset" dialog open, with the chosen target.
    save_target: Option<PresetId>,
    /// "Swap presets" dialog open, with the two chosen.
    swap: Option<(PresetId, PresetId)>,
    /// "IRs & amps" window.
    library: crate::library::Library,
    started: Instant,
    notice: Option<(Tone, String)>,
    show_request: Arc<AtomicBool>,
    backup_dialog: crate::dialog::PendingDialog,
}

impl TankApp {
    pub fn new(ctx: &egui::Context, show_request: Arc<AtomicBool>) -> Self {
        Self {
            dev: Device::spawn(ctx.clone()),
            conn: Conn::Connecting,
            status: None,
            patch: None,
            bank: None,
            loopback: None,
            held: HashMap::new(),
            log: VecDeque::new(),
            paused: false,
            show_monitor: false,
            last_preset: None,
            save_target: None,
            swap: None,
            library: Default::default(),
            started: Instant::now(),
            notice: None,
            show_request,
            backup_dialog: Default::default(),
        }
    }

    fn online(&self) -> bool {
        matches!(self.conn, Conn::Connected(_)) && self.status.is_some()
    }

    fn push_log(&mut self, out: bool, bytes: String, text: String) {
        if self.paused {
            return;
        }
        if self.log.len() == LOG_MAX {
            self.log.pop_front();
        }
        self.log.push_back(LogLine {
            at: self.started.elapsed(),
            out,
            bytes,
            text,
        });
    }

    fn drain_events(&mut self) {
        while let Ok(ev) = self.dev.rx.try_recv() {
            match ev {
                Event::Connected {
                    firmware,
                    bluetooth,
                    bank,
                } => {
                    let via = if bluetooth { "Bluetooth" } else { "USB" };
                    self.conn = Conn::Connected(format!("{firmware} · {via}"));
                    self.bank = bank;
                }
                Event::State {
                    status,
                    patch,
                    loopback,
                } => {
                    if self.status.map(|s| (s.preset, s.mode)) != Some((status.preset, status.mode))
                    {
                        let p = status.preset.map_or("none".into(), |p| p.to_string());
                        let m = if status.mode == Mode::Preset {
                            "P"
                        } else {
                            "L"
                        };
                        self.push_log(
                            false,
                            "READ 0x20000000".into(),
                            format!("Status: preset {p} · mode {m}"),
                        );
                    }
                    if status.preset.is_some() {
                        self.last_preset = status.preset;
                    }
                    self.status = Some(status);
                    self.patch = Some(patch.to_bytes());
                    self.loopback = Some(loopback);
                }
                Event::Disconnected(err) => {
                    self.conn = Conn::Disconnected(err);
                    self.status = None;
                    self.patch = None;
                    self.bank = None;
                    self.loopback = None;
                    self.held.clear();
                }
                Event::BlockProgress(p) => self.library.progress = p,
                Event::Bank(bank) => {
                    self.bank = Some(bank);
                    self.held.clear();
                }
                Event::Log { out, bytes, text } => self.push_log(out, bytes, text),
                Event::Notice(tone, text) => self.notice = Some((tone, text)),
            }
        }
    }

    /// Preset on screen: the pedal's, with recent edits on top.
    fn view(&mut self) -> Option<[u8; PATCH_LEN]> {
        let mut b = self.patch?;
        let now = Instant::now();
        self.held.retain(|_, (_, until)| *until > now);
        for (&o, &(v, _)) in &self.held {
            b[o] = v;
        }
        Some(b)
    }

    fn edit(&mut self, offset: usize, value: u8) {
        self.held.insert(offset, (value, Instant::now() + HOLD));
        self.dev.send(Cmd::SetByte { offset, value });
    }

    fn select(&mut self, p: PresetId) {
        if self.online() {
            self.dev.send(Cmd::Select(p));
        }
    }

    /// Preset playing. In bypass the pedal doesn't say, so the last one seen counts.
    fn active(&self) -> Option<PresetId> {
        let st = self.status?;
        st.preset.or(self.last_preset.filter(|_| self.bypassed()))
    }

    /// Bypass on: letter `ff` after a preset was already seen in the same bank.
    fn bypassed(&self) -> bool {
        self.status.is_some_and(|s| {
            s.preset.is_none() && self.last_preset.is_some_and(|p| p.bank() == s.bank)
        })
    }

    fn live(&self) -> bool {
        self.status.is_some_and(|s| s.mode == Mode::Live)
    }

    fn step_bank(&mut self, d: i32) {
        let (bank, slot) = self
            .active()
            .map_or((0, 0), |p| (p.bank() as i32, p.slot()));
        let b = (bank + d).rem_euclid(preset::BANKS as i32) as u8;
        if let Some(p) = PresetId::from_bank_slot(b, slot) {
            self.select(p);
        }
    }

    /// Footswitch `i` (0–3): in LIVE toggles effects, in PRESET plays the bank's preset.
    fn press(&mut self, i: usize, view: Option<[u8; PATCH_LEN]>) {
        if self.live() {
            if let Some(v) = view {
                let o = LIVE_SWITCHES[i].0;
                self.edit(o, (v[o] == 0) as u8);
            }
        } else {
            let bank = self.active().map_or(0, |p| p.bank());
            if let Some(p) = PresetId::from_bank_slot(bank, i as u8) {
                self.select(p);
            }
        }
    }

    fn backup(&mut self, ctx: &egui::Context) {
        let name = format!("tankg-presets-{}.bin", today());
        self.backup_dialog.start(ctx, move || {
            rfd::FileDialog::new()
                .set_title("Save preset backup")
                .set_file_name(name)
                .add_filter("Tank-G preset bank", &["bin"])
                .save_file()
        });
    }

    fn shortcuts(&mut self, ui: &Ui, view: Option<[u8; PATCH_LEN]>) {
        if !self.online() {
            return;
        }
        let (keys, up, down) = ui.input(|i| {
            if i.modifiers.ctrl || i.modifiers.alt || i.modifiers.command {
                return (None, false, false);
            }
            let k = [Key::Num1, Key::Num2, Key::Num3, Key::Num4]
                .iter()
                .position(|k| i.key_pressed(*k));
            (k, i.key_pressed(Key::PageUp), i.key_pressed(Key::PageDown))
        });
        if let Some(i) = keys {
            self.press(i, view);
        }
        if up {
            self.step_bank(1);
        }
        if down {
            self.step_bank(-1);
        }
    }

    // ------------------------------------------------------------ screens

    fn top_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Tank Studio").font(bold(22.0)).color(ACCENT));
            ui.add_space(16.0);
            let (dot, text) = match &self.conn {
                Conn::Connecting => (WARN, "Looking for the pedal…".to_owned()),
                Conn::Connected(fw) => (OK, fw.clone()),
                Conn::Disconnected(e) => (DANGER, format!("Disconnected: {e}")),
            };
            let (r, _) = ui.allocate_exact_size(vec2(10.0, 10.0), egui::Sense::hover());
            ui.painter().circle_filled(r.center(), 5.0, dot);
            ui.label(RichText::new(text).color(MUTED));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::primary_button(
                    ui,
                    "Back up presets",
                    self.online() && !self.backup_dialog.is_open(),
                )
                .clicked()
                {
                    self.backup(ui.ctx());
                }
                if ui
                    .add(egui::Button::new("Community").min_size(vec2(0.0, 30.0)))
                    .on_hover_text("IRs and amps from community.m-vave.com")
                    .clicked()
                {
                    self.library.open_community();
                }
                if ui
                    .add(egui::Button::new("IRs & amps…").min_size(vec2(0.0, 30.0)))
                    .on_hover_text("Audition and save IRs and amps on the pedal")
                    .clicked()
                {
                    self.library.open = true;
                }
                let label = if self.show_monitor {
                    "Hide MIDI monitor"
                } else {
                    "MIDI monitor"
                };
                if ui
                    .add(egui::Button::new(label).min_size(vec2(0.0, 30.0)))
                    .clicked()
                {
                    self.show_monitor = !self.show_monitor;
                }
            });
        });
    }

    fn notice_bar(&mut self, ui: &mut Ui) {
        let Some((tone, text)) = &self.notice else {
            return;
        };
        let (fg, bg) = match tone {
            Tone::Ok => (OK, ACCENT_SOFT),
            Tone::Danger => (DANGER, DANGER_SOFT),
        };
        let mut close = false;
        egui::Frame::new()
            .fill(bg)
            .corner_radius(8)
            .inner_margin(vec2(12.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(text).color(fg));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        close = ui.small_button("✕").on_hover_text("Dismiss").clicked();
                    });
                });
            });
        if close {
            self.notice = None;
        }
    }

    fn presets(&mut self, ui: &mut Ui) {
        let active = self.active();
        let online = self.online();
        let meta = active
            .map(|p| format!("On the pedal: {p}"))
            .unwrap_or_default();
        let mut open_swap = false;
        card(
            ui,
            "Presets",
            |ui| {
                let b = ui
                    .add_enabled(online, egui::Button::new("Swap presets…"))
                    .on_hover_text("Swap two presets on the pedal");
                open_swap = b.clicked();
                ui.label(RichText::new(meta).color(MUTED).small());
            },
            |ui| {
                ui.set_min_height(BOTTOM_ROW_H);
                egui::Grid::new("preset-grid")
                    .spacing(vec2(6.0, 6.0))
                    .show(ui, |ui| {
                        ui.label("");
                        for bank in 0..preset::BANKS {
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new((bank + 1).to_string())
                                        .font(bold(14.0))
                                        .color(FAINT),
                                )
                            });
                        }
                        ui.end_row();
                        for slot in 0..preset::SLOTS {
                            ui.label(
                                RichText::new(LETTERS[slot as usize])
                                    .font(bold(14.0))
                                    .color(FAINT),
                            );
                            for bank in 0..preset::BANKS {
                                let id = PresetId::from_bank_slot(bank, slot).unwrap();
                                let on = active == Some(id);
                                let b = egui::Button::new(
                                    RichText::new(id.to_string())
                                        .font(semibold(14.0))
                                        .color(if on { ACCENT } else { INK }),
                                )
                                .fill(if on { ACCENT_SOFT } else { CONTROL })
                                .stroke(if on {
                                    Stroke::new(1.0, ACCENT)
                                } else {
                                    Stroke::NONE
                                })
                                .min_size(vec2(46.0, 34.0));
                                if ui
                                    .add_enabled(online, b)
                                    .on_hover_text(format!("Play {id}"))
                                    .clicked()
                                {
                                    self.select(id);
                                }
                            }
                            ui.end_row();
                        }
                    });
                ui.add_space(6.0);
                ui.label(
                    RichText::new("Click to play the preset on the pedal.")
                        .color(FAINT)
                        .small(),
                );
            },
        );
        if open_swap {
            let a = active.unwrap_or(PresetId::new(0).unwrap());
            let b = PresetId::new((a.index() + 1) % preset::COUNT).unwrap();
            self.swap = Some((a, b));
        }
    }

    fn hardware(&mut self, ui: &mut Ui, view: Option<[u8; PATCH_LEN]>) {
        let online = self.online();
        let active = self.active();
        let live = self.live();
        card(
            ui,
            "On the pedal",
            |_| {},
            |ui| {
                ui.set_min_height(BOTTOM_ROW_H);
                ui.horizontal_wrapped(|ui| {
                ui.vertical(|ui| {
                    widgets::led_digit(ui, active.map(|p| p.bank() + 1), 84.0);
                    ui.label(RichText::new("BANK").font(semibold(11.0)).color(FAINT));
                });
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    if ui.add_enabled(online, egui::Button::new("Bank +").min_size(vec2(84.0, 28.0))).on_hover_text("PgUp").clicked() {
                        self.step_bank(1);
                    }
                    if ui.add_enabled(online, egui::Button::new("Bank −").min_size(vec2(84.0, 28.0))).on_hover_text("PgDn").clicked() {
                        self.step_bank(-1);
                    }
                });
                ui.add_space(24.0);
                ui.vertical(|ui| {
                    let name = active.map_or("—".to_owned(), |p| p.to_string());
                    ui.label(RichText::new(name).font(bold(44.0)).color(INK));
                    match self.status {
                        Some(st) => {
                            ui.horizontal(|ui| {
                                for (m, label) in [(Mode::Preset, "PRESET"), (Mode::Live, "LIVE")] {
                                    let on = st.mode == m;
                                    let b = egui::Button::new(RichText::new(label).font(semibold(12.0)).color(if on { ON_ACCENT } else { INK }))
                                        .fill(if on { ACCENT } else { CONTROL })
                                        .min_size(vec2(72.0, 26.0));
                                    if ui.add(b).clicked() && !on {
                                        self.dev.send(Cmd::SetMode(m));
                                    }
                                }
                            });
                            ui.label(
                                RichText::new(if live { "Footswitches turn effects on and off." } else { "Footswitches play the bank's presets." })
                                    .color(MUTED),
                            );
                        }
                        None => {
                            widgets::tag(ui, "No reading", MUTED, CONTROL);
                        }
                    }
                    if let Some(lb) = self.loopback {
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            let r = widgets::switch(ui, lb, ACCENT, online).on_hover_text(
                                "Sends the PC's audio back through the USB input, along with the guitar. Off avoids feedback with Reaper.",
                            );
                            if r.clicked() {
                                self.dev.send(Cmd::SetLoopback(!lb));
                            }
                            ui.label(format!("Loopback · {}", if lb { "on" } else { "off" }));
                        });
                    }
                    if self.status.is_some() {
                        let by = self.bypassed();
                        ui.horizontal(|ui| {
                            let r = widgets::switch(ui, by, RED, online && self.last_preset.is_some())
                                .on_hover_text("Plays the preset with no effects. Turning it off goes back to the saved preset (unsaved edits are lost).");
                            if r.clicked() {
                                self.dev.send(Cmd::Bypass(!by));
                            }
                            ui.label(if by { RichText::new("Bypass · on").color(RED) } else { RichText::new("Bypass · off") });
                        });
                    }
                });
                ui.add_space(24.0);
                for i in 0..4 {
                    let r = if live && let Some(v) = view {
                        let (o, label, color) = LIVE_SWITCHES[i];
                        widgets::footswitch(ui, LETTERS[i], label, v[o] != 0, color, online)
                            .on_hover_text(format!("Turn {label} {} (key {})", if v[o] != 0 { "off" } else { "on" }, i + 1))
                    } else {
                        let bank = active.map_or(0, |p| p.bank());
                        let id = PresetId::from_bank_slot(bank, i as u8).unwrap();
                        widgets::footswitch(ui, LETTERS[i], &id.to_string(), active == Some(id), ACCENT, online)
                            .on_hover_text(format!("Play {id} (key {})", i + 1))
                    };
                    if r.clicked() {
                        self.press(i, view);
                    }
                    ui.add_space(8.0);
                }
            });
            },
        );
    }

    fn current_preset(&mut self, ui: &mut Ui, view: Option<[u8; PATCH_LEN]>, board_max_h: f32) {
        let active = self.active();
        let saved = active
            .and_then(|p| self.bank.as_ref()?.get(p.index() as usize))
            .map(Patch::to_bytes);
        let modified =
            matches!((view, saved), (Some(v), Some(s)) if v[..off::KNOWN] != s[..off::KNOWN]);
        let mut discard = false;
        let mut open_save = false;
        let online = self.online();
        let title = active.map_or("Current preset".to_owned(), |p| {
            format!("Current preset · {p}")
        });
        card(
            ui,
            &title,
            |ui| {
                open_save = view.is_some()
                    && widgets::primary_button(ui, "Save…", online)
                        .on_hover_text("Store this sound in one of the pedal's presets")
                        .clicked();
                if active.is_some() {
                    discard = ui
                        .add_enabled(modified, egui::Button::new("Discard edits"))
                        .on_hover_text(
                            "Plays the preset again: the pedal goes back to what is saved",
                        )
                        .clicked();
                }
                if view.is_some() && saved.is_some() {
                    if modified {
                        widgets::tag(ui, "Modified · not saved", WARN, WARN_SOFT);
                    } else {
                        widgets::tag(ui, "Same as saved", OK, ACCENT_SOFT);
                    }
                }
            },
            |ui| {
                let Some(v) = view else {
                    ui.label(RichText::new("Connect the pedal to see the preset.").color(MUTED));
                    return;
                };
                for (o, value) in
                    pedalboard::show(ui, &v, saved.as_ref(), self.online(), board_max_h)
                {
                    self.edit(o, value);
                }
                ui.add_space(8.0);
                let mut hint = "Turn the knobs and press the pedals' footswitches. Changes go to the pedal right away but are not saved: switching presets goes back to the saved one.".to_owned();
                if !self.live() {
                    hint += " In PRESET mode the physical knobs don't edit, but the app's do.";
                }
                hint += " Amp names are the factory ones. MASTER is global and not stored in the preset.";
                ui.label(RichText::new(hint).color(FAINT).small());
            },
        );
        if open_save {
            self.save_target = Some(active.unwrap_or(PresetId::new(0).unwrap()));
        }
        if discard && let Some(p) = active {
            self.held.clear();
            self.select(p);
        }
    }

    fn monitor(&mut self, ui: &mut Ui) {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("MIDI monitor").font(semibold(14.0)));
            ui.label(
                RichText::new(format!("{} messages", self.log.len()))
                    .color(FAINT)
                    .small(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("Clear").clicked() {
                    self.log.clear();
                }
                if ui
                    .small_button(if self.paused { "Resume" } else { "Pause" })
                    .clicked()
                {
                    self.paused = !self.paused;
                }
            });
        });
        ui.add_space(4.0);
        egui::Frame::new()
            .fill(DISPLAY_BG)
            .corner_radius(8)
            .inner_margin(8)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        let mono = egui::FontId::monospace(12.0);
                        for l in &self.log {
                            ui.horizontal(|ui| {
                                let t = l.at.as_secs_f32();
                                ui.label(
                                    RichText::new(format!("{:>8.3}s", t))
                                        .font(mono.clone())
                                        .color(FAINT),
                                );
                                let (arrow, c) = if l.out {
                                    ("→", ACCENT)
                                } else {
                                    ("←", GREEN)
                                };
                                ui.label(RichText::new(arrow).font(mono.clone()).color(c));
                                ui.add_sized(
                                    vec2(220.0, 14.0),
                                    egui::Label::new(
                                        RichText::new(&l.bytes).font(mono.clone()).color(MUTED),
                                    )
                                    .truncate(),
                                );
                                ui.label(RichText::new(&l.text).color(INK));
                            });
                        }
                    });
            });
    }
}

impl TankApp {
    fn dialogs(&mut self, ctx: &egui::Context, view: Option<[u8; PATCH_LEN]>) {
        if self.library.open
            && let Some(cmd) = self.library.show(ctx, self.online())
        {
            self.dev.send(cmd);
        }
        if let Some(mut target) = self.save_target {
            let mut close = false;
            let resp = egui::Modal::new(egui::Id::new("save")).show(ctx, |ui| {
                ui.set_width(460.0);
                ui.label(RichText::new("Save preset").font(bold(18.0)));
                ui.add_space(6.0);
                ui.label("The sound on the pedal right now will be stored in the chosen preset, replacing what is there.");
                ui.add_space(10.0);
                preset_picker(ui, "save-grid", &mut target, self.active());
                ui.add_space(10.0);
                if Some(target) != self.active() {
                    ui.label(RichText::new(format!("Careful: the current {target} will be replaced.")).color(WARN));
                }
                ui.label(RichText::new(format!("Before writing, the whole bank is backed up to {}.", tankg_device::backup_dir().display())).color(FAINT).small());
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if widgets::primary_button(ui, &format!("Save to {target}"), view.is_some() && self.online()).clicked()
                        && let Some(rec) = view
                    {
                        self.dev.send(Cmd::Save { target, rec });
                        close = true;
                    }
                    if ui.button("Cancel").clicked() {
                        close = true;
                    }
                });
            });
            self.save_target = (!close && !resp.should_close()).then_some(target);
        }

        if let Some((mut a, mut b)) = self.swap {
            let mut close = false;
            let resp = egui::Modal::new(egui::Id::new("swap")).show(ctx, |ui| {
                ui.set_width(460.0);
                ui.label(RichText::new("Swap presets").font(bold(18.0)));
                ui.add_space(6.0);
                ui.label("The two presets swap places on the pedal.");
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    preset_combo(ui, "swap-a", &mut a);
                    ui.label(RichText::new("⇄").font(bold(18.0)).color(ACCENT));
                    preset_combo(ui, "swap-b", &mut b);
                });
                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!(
                        "Before writing, the whole bank is backed up to {}.",
                        tankg_device::backup_dir().display()
                    ))
                    .color(FAINT)
                    .small(),
                );
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if widgets::primary_button(
                        ui,
                        &format!("Swap {a} ⇄ {b}"),
                        a != b && self.online(),
                    )
                    .clicked()
                    {
                        self.dev.send(Cmd::Swap(a, b));
                        close = true;
                    }
                    if ui.button("Cancel").clicked() {
                        close = true;
                    }
                });
            });
            self.swap = (!close && !resp.should_close()).then_some((a, b));
        }
    }
}

/// 9 × 4 grid to pick a preset.
fn preset_picker(ui: &mut Ui, id: &str, value: &mut PresetId, active: Option<PresetId>) {
    egui::Grid::new(id).spacing(vec2(4.0, 4.0)).show(ui, |ui| {
        for slot in 0..preset::SLOTS {
            for bank in 0..preset::BANKS {
                let p = PresetId::from_bank_slot(bank, slot).unwrap();
                let on = *value == p;
                let mut text = RichText::new(p.to_string())
                    .font(semibold(13.0))
                    .color(if on { ON_ACCENT } else { INK });
                if active == Some(p) && !on {
                    text = text.color(ACCENT);
                }
                let b = egui::Button::new(text)
                    .fill(if on { ACCENT } else { CONTROL })
                    .min_size(vec2(42.0, 28.0));
                if ui
                    .add(b)
                    .on_hover_text(if active == Some(p) { "Playing now" } else { "" })
                    .clicked()
                {
                    *value = p;
                }
            }
            ui.end_row();
        }
    });
}

fn preset_combo(ui: &mut Ui, id: &str, value: &mut PresetId) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(value.to_string())
        .width(80.0)
        .show_ui(ui, |ui| {
            for i in 0..preset::COUNT {
                let p = PresetId::new(i).unwrap();
                ui.selectable_value(value, p, p.to_string());
            }
        });
}

/// (byte, name, color) of each footswitch in LIVE mode, A–D.
const LIVE_SWITCHES: [(usize, &str, Color32); 4] = [
    (off::REVERB_ON, "REVERB", PURPLE),
    (off::DELAY_ON, "DELAY", BLUE),
    (off::MOD_ON, "MOD", GREEN),
    (off::AMP_ON, "AMP", RED),
];

impl eframe::App for TankApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_events();
        if self.show_request.swap(false, Ordering::Relaxed) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        if let Some(path) = self.backup_dialog.take() {
            self.dev.send(Cmd::Backup(path));
        }
        let view = self.view();
        self.shortcuts(ui, view);
        let g = GAP as i8;
        let frame = |m: egui::Margin| egui::Frame::new().fill(GROUND).inner_margin(m);
        let pad = frame(egui::Margin {
            left: g,
            right: g,
            top: g,
            bottom: 0,
        });

        egui::Panel::top("top")
            .frame(pad)
            .show_separator_line(false)
            .show(ui, |ui| {
                self.top_bar(ui);
                if self.notice.is_some() {
                    ui.add_space(8.0);
                    self.notice_bar(ui);
                }
            });
        if self.show_monitor {
            egui::Panel::bottom("monitor")
                .frame(frame(egui::Margin {
                    left: g,
                    right: g,
                    top: 0,
                    bottom: g,
                }))
                .resizable(true)
                .default_size(170.0)
                .size_range(90.0..=500.0)
                .show(ui, |ui| self.monitor(ui));
        }
        egui::CentralPanel::no_frame()
            .frame(frame(egui::Margin::same(g)))
            .show(ui, |ui| {
                // Pedals as big as fits without scrolling, leaving room for the
                // "On the pedal" card. Narrow or portrait windows stack the two
                // lower cards (the presets list is then below the fold).
                let vertical = ui.available_width() < VERTICAL_BELOW
                    || ui.available_height() > ui.available_width();
                let board_max_h = (ui.available_height() - BOTTOM_ROW_H - RESERVED_H).max(180.0);
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        self.current_preset(ui, view, board_max_h);
                        ui.add_space(GAP - ui.spacing().item_spacing.y);
                        if vertical {
                            self.hardware(ui, view);
                            ui.add_space(GAP - ui.spacing().item_spacing.y);
                            self.presets(ui);
                        } else {
                            ui.horizontal_top(|ui| {
                                ui.spacing_mut().item_spacing.x = GAP;
                                let w = (ui.available_width() - GAP) / 2.0;
                                let col = egui::Layout::top_down(egui::Align::Min);
                                ui.allocate_ui_with_layout(vec2(w, 0.0), col, |ui| {
                                    self.hardware(ui, view)
                                });
                                ui.allocate_ui_with_layout(vec2(w, 0.0), col, |ui| {
                                    self.presets(ui)
                                });
                            });
                        }
                    });
            });

        self.dialogs(ui.ctx(), view);

        if !self.held.is_empty() {
            ui.ctx().request_repaint_after(Duration::from_millis(250));
        }
    }
}

/// Today's date (UTC) as YYYY-MM-DD, for the backup file name.
fn today() -> String {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() / 86_400) as i64;
    // Howard Hinnant, civil_from_days
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + (m <= 2) as i64;
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    #[test]
    fn date_format() {
        let t = super::today();
        assert_eq!(t.len(), 10);
        assert!(t.starts_with("20"));
    }
}
