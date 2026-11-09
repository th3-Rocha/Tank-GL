//! "IRs & amps" window: pick an IR (WAV) or amp (.am4data) from the Community,
//! a file or the factory set, audition it on the pedal without saving, and save
//! it to a slot. Factory ones come from the installed M-EFCS, so you can always go back.

use egui::{Align, Color32, CornerRadius, Layout, RichText, Stroke, Ui, vec2};
use tankg_proto::block::{self, Kind};

use crate::community::{self, Community, FileKind, Fit, Item, RemoteFile, Reply};
use crate::device::Cmd;
use crate::pedalboard::AMP_NAMES;
use crate::theme::*;
use crate::widgets::{self, bold, semibold};

const IR_NAMES: [&str; 8] = [
    "AC-SeVin",
    "JVM 1960 57",
    "MESA 412",
    "WANGS112",
    "VOX AC30",
    "HIFI YAMAHA",
    "EGNL 412",
    "V30 MD421",
];
const SIDE_W: f32 = 340.0;
const GAP: f32 = 16.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Source {
    Community,
    File,
    Factory,
}

struct Loaded {
    rec: Vec<u8>,
    label: String,
    detail: String,
    /// Compatibility warning (long IR cut, etc.).
    warn: Option<String>,
}

pub struct Library {
    pub open: bool,
    file_dialog: crate::dialog::PendingDialog,
    kind: Kind,
    source: Source,
    loaded: Option<Loaded>,
    slot: u8,
    /// What is loaded was already sent and is playing on the pedal.
    auditioned: bool,
    confirm_save: bool,
    error: Option<String>,
    /// M-EFCS factory bank, read the first time it is needed.
    factory: Option<Option<Vec<u8>>>,
    pub progress: Option<(usize, usize)>,
    /// community.m-vave.com client, created on the first search.
    community: Option<Community>,
    query: String,
    results: Vec<Item>,
    /// (query, total on the site) of the last reply.
    searched: Option<(String, u64)>,
    /// Search as soon as the window opens.
    auto_search: bool,
}

impl Default for Library {
    fn default() -> Self {
        Self {
            open: false,
            file_dialog: Default::default(),
            kind: Kind::Cab,
            source: Source::Community,
            loaded: None,
            slot: 1,
            auditioned: false,
            confirm_save: false,
            error: None,
            factory: None,
            progress: None,
            community: None,
            query: String::new(),
            results: Vec::new(),
            searched: None,
            auto_search: false,
        }
    }
}

fn kind_color(k: Kind) -> Color32 {
    match k {
        Kind::Cab => YELLOW,
        Kind::Amp => RED,
    }
}

impl Library {
    /// Opens showing the Community's latest uploads.
    pub fn open_community(&mut self) {
        self.open = true;
        self.source = Source::Community;
        if self.results.is_empty() {
            self.auto_search = true;
        }
    }

    fn file_kind(&self) -> FileKind {
        match self.kind {
            Kind::Amp => FileKind::Amp,
            Kind::Cab => FileKind::Ir,
        }
    }

    fn factory_bank(&mut self) -> Option<&Vec<u8>> {
        self.factory
            .get_or_insert_with(|| {
                tankg_device::find_factory_bank().and_then(|p| std::fs::read(p).ok())
            })
            .as_ref()
    }

    fn slot_name(&self, slot: u8) -> &'static str {
        let i = slot as usize - 1;
        match self.kind {
            Kind::Cab => IR_NAMES.get(i).copied().unwrap_or("?"),
            Kind::Amp => AMP_NAMES.get(i).copied().unwrap_or("?"),
        }
    }

    fn set_loaded(&mut self, l: Loaded) {
        self.loaded = Some(l);
        self.auditioned = false;
        self.confirm_save = false;
        self.error = None;
    }

    /// Turns what came from a file (disk or community) into a record.
    fn load_bytes(&mut self, label: String, bytes: Vec<u8>) {
        let res = match self.kind {
            Kind::Cab => block::wav_to_cab(&bytes).map(|(s, info)| {
                let ms = info.frames as f64 * 1000.0 / info.rate as f64;
                let used = info.frames as f64 * block::CAB_RATE as f64 / info.rate as f64;
                let warn = (used > block::CAB_SAMPLES as f64 * 1.02).then(|| {
                    format!("{ms:.0} ms IR: the Tank-G only uses the first 46 ms. The rest is cut, so reverbs and long rooms won't sound like the original.")
                });
                (block::cab_record(&s, block::CAB_DEFAULT_LEVEL), info.describe(), warn)
            }),
            Kind::Amp => block::check_record(Kind::Amp, &bytes).map(|()| {
                let detail = format!("amp {:?} · {} bytes", block::record_name(&bytes), bytes.len());
                (bytes, detail, None)
            }),
        };
        match res {
            Ok((rec, detail, warn)) => self.set_loaded(Loaded {
                rec,
                label,
                detail,
                warn,
            }),
            Err(e) => self.error = Some(format!("{label}: {e}")),
        }
    }

    fn open_file(&mut self, ctx: &egui::Context) {
        let (filter, ext): (&str, &[&str]) = match self.kind {
            Kind::Cab => ("IR (WAV)", &["wav", "WAV"]),
            Kind::Amp => ("Tank-G amp (.am4data)", &["am4data", "am4Data"]),
        };
        self.file_dialog.start(ctx, move || {
            rfd::FileDialog::new().add_filter(filter, ext).pick_file()
        });
    }

    fn load_path(&mut self, path: &std::path::Path) {
        let label = path.file_name().map_or_else(
            || path.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        );
        match std::fs::read(path) {
            Ok(bytes) => self.load_bytes(label, bytes),
            Err(e) => self.error = Some(format!("{label}: {e}")),
        }
    }

    fn load_factory(&mut self, slot: u8) {
        let kind = self.kind;
        let rec = self
            .factory_bank()
            .and_then(|b| block::factory_record(b, kind, slot));
        match rec {
            Some(rec) => {
                let name = block::record_name(&rec);
                self.set_loaded(Loaded {
                    rec,
                    label: format!("{name} (factory)"),
                    detail: format!("slot {slot} original"),
                    warn: None,
                });
                self.slot = slot;
            }
            None => self.error = Some("Could not read the M-EFCS factory bank.".into()),
        }
    }

    fn search(&mut self, ctx: &egui::Context) {
        let kind = self.file_kind();
        self.community
            .get_or_insert_with(|| Community::spawn(ctx.clone()))
            .search(self.query.clone(), kind);
    }

    fn poll_community(&mut self) {
        let Some(c) = &mut self.community else { return };
        let mut got = Vec::new();
        while let Ok(r) = c.rx.try_recv() {
            c.busy = false;
            got.push(r);
        }
        let current = self.file_kind();
        for r in got {
            match r {
                // Reply to a search from the other tab: drop it.
                Reply::Results { kind, .. } | Reply::File { kind, .. } if kind != current => {}
                Reply::Results {
                    query,
                    items,
                    total,
                    ..
                } => {
                    self.results = items;
                    self.searched = Some((query, total));
                    self.error = None;
                }
                Reply::File { label, bytes, .. } => self.load_bytes(label, bytes),
                Reply::Error(e) => self.error = Some(e),
            }
        }
    }

    fn switch_kind(&mut self, k: Kind) {
        let searched = self.searched.is_some();
        *self = Self {
            open: true,
            kind: k,
            source: self.source,
            factory: self.factory.take(),
            community: self.community.take(),
            query: std::mem::take(&mut self.query),
            auto_search: searched || self.source == Source::Community,
            ..Self::default()
        };
    }

    /// Draws the window. Returns the command for the pedal, if any.
    pub fn show(&mut self, ctx: &egui::Context, online: bool) -> Option<Cmd> {
        self.poll_community();
        if let Some(path) = self.file_dialog.take() {
            self.load_path(&path);
        }
        if std::mem::take(&mut self.auto_search) {
            self.search(ctx);
        }
        // dragging a file onto the window
        let dropped: Vec<_> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_path_buf())
                .collect()
        });
        if let Some(p) = dropped.first() {
            let ext = p
                .extension()
                .map(|e| e.to_string_lossy().to_ascii_lowercase())
                .unwrap_or_default();
            if ext == "am4data" && self.kind != Kind::Amp {
                self.switch_kind(Kind::Amp);
            } else if ext == "wav" && self.kind != Kind::Cab {
                self.switch_kind(Kind::Cab);
            }
            self.auto_search = false;
            self.source = Source::File;
            self.load_path(p);
        }

        let screen = ctx.content_rect();
        let w = (screen.width() - 64.0).clamp(760.0, 1280.0);
        let h = (screen.height() - 64.0).clamp(540.0, 880.0);
        let busy = self.progress.is_some();
        let mut cmd = None;

        let resp = egui::Modal::new(egui::Id::new("library"))
            .frame(
                egui::Frame::new()
                    .fill(CARD)
                    .stroke(Stroke::new(1.0, LINE_STRONG))
                    .corner_radius(16)
                    .inner_margin(22),
            )
            .show(ctx, |ui| {
                ui.set_width(w);
                ui.set_height(h);
                self.header(ui);
                ui.add_space(14.0);
                let body_h = ui.available_height();
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = GAP;
                    let left_w = ui.available_width() - SIDE_W - GAP;
                    ui.allocate_ui_with_layout(
                        vec2(left_w, body_h),
                        Layout::top_down(Align::Min),
                        |ui| {
                            ui.set_min_size(vec2(left_w, body_h));
                            self.sources(ui, busy);
                        },
                    );
                    ui.allocate_ui_with_layout(
                        vec2(SIDE_W, body_h),
                        Layout::top_down(Align::Min),
                        |ui| {
                            cmd = self.side(ui, online, busy, body_h);
                        },
                    );
                });
            });
        if resp.should_close() && !busy {
            self.open = false;
        }
        cmd
    }

    fn header(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("IRs & amps").font(bold(26.0)).color(INK));
                ui.label(RichText::new("Pick an IR or amp, hear it on the pedal and, if you like it, save it to a slot.").color(MUTED));
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(egui::Button::new(RichText::new("✕  Close").font(semibold(14.0))).min_size(vec2(110.0, 36.0))).clicked() {
                    self.open = false;
                }
                ui.add_space(12.0);
                for k in [Kind::Amp, Kind::Cab] {
                    let on = self.kind == k;
                    let label = if k == Kind::Cab { "IR · cab" } else { "AMP" };
                    let c = kind_color(k);
                    let b = egui::Button::new(RichText::new(label).font(bold(15.0)).color(if on { Color32::BLACK } else { INK }))
                        .fill(if on { c } else { CONTROL })
                        .stroke(if on { Stroke::NONE } else { Stroke::new(1.0, c.gamma_multiply(0.5)) })
                        .corner_radius(8)
                        .min_size(vec2(130.0, 36.0));
                    if ui.add_enabled(self.progress.is_none(), b).clicked() && !on {
                        self.switch_kind(k);
                    }
                }
            });
        });
    }

    // -------------------------------------------------------- left column

    fn sources(&mut self, ui: &mut Ui, busy: bool) {
        ui.horizontal(|ui| {
            for (s, label) in [
                (Source::Community, "Community"),
                (Source::File, "File"),
                (Source::Factory, "Factory"),
            ] {
                let on = self.source == s;
                let text = RichText::new(label).font(semibold(15.0)).color(if on {
                    ACCENT
                } else {
                    MUTED
                });
                let r = ui.add(
                    egui::Button::new(text)
                        .frame(false)
                        .min_size(vec2(0.0, 30.0)),
                );
                if on {
                    let rect = r.rect;
                    ui.painter().line_segment(
                        [rect.left_bottom(), rect.right_bottom()],
                        Stroke::new(2.5, ACCENT),
                    );
                }
                if r.clicked() && !on {
                    self.source = s;
                    if s == Source::Community && self.searched.is_none() {
                        self.search(ui.ctx());
                    }
                }
                ui.add_space(10.0);
            }
        });
        ui.add_space(10.0);
        if let Some(e) = self.error.clone() {
            egui::Frame::new()
                .fill(DANGER_SOFT)
                .corner_radius(8)
                .inner_margin(10)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(RichText::new(e).color(DANGER));
                });
            ui.add_space(8.0);
        }
        match self.source {
            Source::Community => self.community_ui(ui, busy),
            Source::File => self.file_ui(ui, busy),
            Source::Factory => self.factory_ui(ui, busy),
        }
    }

    fn community_ui(&mut self, ui: &mut Ui, busy: bool) {
        let kind = self.file_kind();
        let c_busy = self.community.as_ref().is_some_and(|c| c.busy);
        ui.horizontal(|ui| {
            let hint = if self.kind == Kind::Cab {
                "Search IRs (e.g. mesa, v30, 412, greenback)"
            } else {
                "Search amps (e.g. marshall, 5150, plexi, fender)"
            };
            let field_w = (ui.available_width() - 200.0).max(200.0);
            let r = ui.add_sized(
                vec2(field_w, 38.0),
                egui::TextEdit::singleline(&mut self.query)
                    .hint_text(hint)
                    .font(egui::FontId::proportional(15.0))
                    .vertical_align(Align::Center),
            );
            let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let go = widgets::primary_button(ui, "Search", !busy && !c_busy).clicked();
            if (go || enter) && !c_busy {
                self.search(ui.ctx());
            }
            if c_busy {
                ui.spinner();
            }
        });
        ui.horizontal(|ui| {
            if let Some((q, total)) = &self.searched {
                let q = if q.trim().is_empty() {
                    "latest".to_owned()
                } else {
                    format!("{q:?}")
                };
                ui.label(
                    RichText::new(format!(
                        "{} {} for the Tank-G · {} posts on the site · {q}",
                        self.results.len(),
                        if self.kind == Kind::Cab {
                            "IRs"
                        } else {
                            "amps"
                        },
                        total
                    ))
                    .color(FAINT),
                );
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.hyperlink_to(
                    RichText::new("open community.m-vave.com ↗").color(ACCENT),
                    community::SITE,
                );
            });
        });
        ui.add_space(6.0);
        if self.results.is_empty() {
            let msg = if c_busy {
                "Searching…"
            } else if self.searched.is_some() {
                "Nothing found. Try another word."
            } else {
                ""
            };
            ui.add_space(30.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(msg).color(FAINT).font(semibold(15.0)))
            });
            return;
        }
        let mut pick = None;
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                for it in &self.results {
                    egui::Frame::new()
                        .fill(RAISED)
                        .corner_radius(10)
                        .stroke(Stroke::new(1.0, LINE))
                        .inner_margin(egui::Margin::symmetric(14, 10))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.label(RichText::new(&it.title).font(semibold(15.0)).color(INK));
                            ui.label(
                                RichText::new(format!(
                                    "by {} · {} downloads",
                                    it.author, it.downloads
                                ))
                                .color(FAINT)
                                .small(),
                            );
                            ui.add_space(4.0);
                            for f in &it.files {
                                ui.horizontal(|ui| {
                                    let fit = f.fit(kind);
                                    let usable = !(kind == FileKind::Amp && fit == Fit::Bad);
                                    let b = egui::Button::new(
                                        RichText::new("Use")
                                            .font(semibold(13.0))
                                            .color(if usable { ON_ACCENT } else { FAINT }),
                                    )
                                    .fill(if usable { ACCENT } else { CONTROL })
                                    .min_size(vec2(64.0, 26.0));
                                    if ui.add_enabled(usable && !busy && !c_busy, b).clicked() {
                                        pick = Some(f.clone());
                                    }
                                    fit_badge(ui, fit, kind);
                                    ui.label(RichText::new(&f.label).color(MUTED));
                                    ui.label(RichText::new(&f.size).color(FAINT).small());
                                });
                            }
                        });
                    ui.add_space(8.0);
                }
            });
        if let Some(f) = pick {
            self.download(kind, f);
        }
    }

    fn download(&mut self, kind: FileKind, f: RemoteFile) {
        if let Some(c) = &mut self.community {
            c.download(kind, f);
        }
    }

    fn file_ui(&mut self, ui: &mut Ui, busy: bool) {
        let (what, formats) = match self.kind {
            Kind::Cab => (
                "an IR (.wav)",
                "Mono or stereo WAV, 16/24/32-bit or float, any rate: the app converts it to 44.1 kHz and 2048 samples.",
            ),
            Kind::Amp => (
                "an amp (.am4data)",
                ".am4data files up to 8 KB, the Tank-G's format. Those for other pedals (.am2/.am3data, 14 KB) don't work.",
            ),
        };
        let hovering = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
        let frame = egui::Frame::new()
            .fill(if hovering { ACCENT_SOFT } else { SURFACE })
            .stroke(Stroke::new(
                1.5,
                if hovering { ACCENT } else { LINE_STRONG },
            ))
            .corner_radius(14)
            .inner_margin(30);
        frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(format!("Drop {what} here"))
                        .font(bold(20.0))
                        .color(INK),
                );
                ui.add_space(6.0);
                ui.label(RichText::new("or").color(FAINT));
                ui.add_space(6.0);
                if ui
                    .add_enabled(
                        !busy && !self.file_dialog.is_open(),
                        egui::Button::new(RichText::new("Choose file…").font(semibold(15.0)))
                            .min_size(vec2(200.0, 40.0)),
                    )
                    .clicked()
                {
                    self.open_file(ui.ctx());
                }
                ui.add_space(12.0);
                ui.label(RichText::new(formats).color(MUTED));
            });
        });
    }

    fn factory_ui(&mut self, ui: &mut Ui, busy: bool) {
        if self.factory_bank().is_none() {
            ui.label(RichText::new("The factory originals come from M-EFCS. Install M-EFCS (Wine) to use this option.").color(MUTED));
            return;
        }
        ui.label(RichText::new("To undo: save the original back to the same slot.").color(MUTED));
        ui.add_space(8.0);
        let mut pick = None;
        let cols = 2;
        let col_w = (ui.available_width() - 10.0) / cols as f32;
        egui::Grid::new("factory")
            .num_columns(cols)
            .spacing(vec2(10.0, 10.0))
            .show(ui, |ui| {
                for s in 1..=self.kind.slots() {
                    egui::Frame::new()
                        .fill(RAISED)
                        .corner_radius(10)
                        .stroke(Stroke::new(1.0, LINE))
                        .inner_margin(12)
                        .show(ui, |ui| {
                            ui.set_width(col_w - 26.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(s.to_string())
                                        .font(bold(20.0))
                                        .color(kind_color(self.kind)),
                                );
                                ui.label(
                                    RichText::new(self.slot_name(s))
                                        .font(semibold(15.0))
                                        .color(INK),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui
                                        .add_enabled(
                                            !busy,
                                            egui::Button::new("Use").min_size(vec2(64.0, 26.0)),
                                        )
                                        .clicked()
                                    {
                                        pick = Some(s);
                                    }
                                });
                            });
                        });
                    if s % cols as u8 == 0 {
                        ui.end_row();
                    }
                }
            });
        if let Some(s) = pick {
            self.load_factory(s);
        }
    }

    // -------------------------------------------------------- right panel

    fn side(&mut self, ui: &mut Ui, online: bool, busy: bool, body_h: f32) -> Option<Cmd> {
        let mut cmd = None;
        egui::Frame::new().fill(RAISED).stroke(Stroke::new(1.0, LINE)).corner_radius(14).inner_margin(18).show(ui, |ui| {
            ui.set_width(SIDE_W - 36.0);
            ui.set_min_height(body_h - 36.0);
            let color = kind_color(self.kind);

            ui.label(RichText::new("SELECTED").font(semibold(11.0)).color(FAINT));
            ui.add_space(4.0);
            match &self.loaded {
                Some(l) => {
                    ui.label(RichText::new(&l.label).font(bold(17.0)).color(INK));
                    ui.label(RichText::new(&l.detail).color(MUTED).small());
                    if let Some(w) = &l.warn {
                        ui.add_space(6.0);
                        egui::Frame::new().fill(WARN_SOFT).corner_radius(8).inner_margin(8).show(ui, |ui| {
                            ui.label(RichText::new(w).color(WARN).small());
                        });
                    }
                    if self.auditioned {
                        ui.add_space(4.0);
                        ui.label(RichText::new("● playing on the pedal (not saved)").color(OK).small());
                    }
                }
                None => {
                    ui.label(RichText::new("Nothing chosen yet").font(semibold(15.0)).color(FAINT));
                    ui.label(RichText::new("Use a Community item, a file or a factory original.").color(FAINT).small());
                }
            }

            ui.add_space(18.0);
            ui.label(RichText::new("LEVEL").font(semibold(11.0)).color(FAINT));
            let enabled = self.loaded.is_some() && !busy;
            let mut level = self.loaded.as_ref().map_or(0, |l| l.rec[19]);
            ui.spacing_mut().slider_width = SIDE_W - 120.0;
            let r = ui.add_enabled(enabled, egui::Slider::new(&mut level, 0..=255));
            if r.changed()
                && let Some(l) = &mut self.loaded
            {
                l.rec[19] = level;
                if self.auditioned {
                    cmd = Some(Cmd::BlockLevel(self.kind, level));
                }
            }

            ui.add_space(18.0);
            ui.label(RichText::new("SLOT ON THE PEDAL").font(semibold(11.0)).color(FAINT));
            ui.add_space(4.0);
            egui::Grid::new("slots").num_columns(2).spacing(vec2(6.0, 6.0)).show(ui, |ui| {
                for s in 1..=self.kind.slots() {
                    let on = self.slot == s;
                    let text = RichText::new(format!("{s}  {}", self.slot_name(s))).font(semibold(13.0)).color(if on { Color32::BLACK } else { INK });
                    let b = egui::Button::new(text).fill(if on { color } else { CONTROL }).corner_radius(6).min_size(vec2((SIDE_W - 48.0) / 2.0, 30.0));
                    if ui.add_enabled(!busy, b).on_hover_text("Factory name of the slot; if you saved something else there, that is what it holds").clicked() {
                        self.slot = s;
                        self.confirm_save = false;
                    }
                    if s % 2 == 0 {
                        ui.end_row();
                    }
                }
            });

            // actions at the bottom of the panel
            ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                let full = vec2(ui.available_width(), 44.0);
                if let Some((done, total)) = self.progress {
                    ui.add_sized(full, egui::ProgressBar::new(done as f32 / total.max(1) as f32).text(format!("sending {done} / {total} bytes")));
                    return;
                }
                let Some(rec) = self.loaded.as_ref().map(|l| l.rec.clone()) else { return };
                if self.confirm_save {
                    if ui.add_sized(full, egui::Button::new("Cancel")).clicked() {
                        self.confirm_save = false;
                    }
                    let save = egui::Button::new(RichText::new(format!("Yes, save to slot {}", self.slot)).font(bold(15.0)).color(Color32::BLACK)).fill(WARN);
                    if ui.add_enabled_ui(online, |ui| ui.add_sized(full, save)).inner.clicked() {
                        cmd = Some(Cmd::Block { kind: self.kind, rec: rec.clone(), slot: Some(self.slot) });
                        self.confirm_save = false;
                        self.auditioned = true;
                    }
                    ui.label(
                        RichText::new(format!(
                            "Replaces the {} in slot {} for every preset that uses that slot. The original can be restored from the Factory tab.",
                            self.kind.name(),
                            self.slot
                        ))
                        .color(WARN),
                    );
                } else {
                    let save = egui::Button::new(RichText::new(format!("Save to slot {}…", self.slot)).font(semibold(15.0))).corner_radius(8);
                    if ui.add_enabled_ui(online, |ui| ui.add_sized(full, save)).inner.clicked() {
                        self.confirm_save = true;
                    }
                    let play = egui::Button::new(RichText::new("▶  Play on the pedal").font(bold(16.0)).color(ON_ACCENT)).fill(ACCENT).corner_radius(8);
                    let r = ui.add_enabled_ui(online, |ui| ui.add_sized(full, play)).inner.on_hover_text("Plays without saving. Switching presets goes back to what is saved.");
                    if r.clicked() {
                        cmd = Some(Cmd::Block { kind: self.kind, rec, slot: None });
                        self.auditioned = true;
                    }
                }
            });
        });
        cmd
    }
}

fn fit_badge(ui: &mut Ui, fit: Fit, kind: FileKind) {
    let (text, fg, bg) = match (fit, kind) {
        (Fit::Good, _) => ("compatible", OK, ACCENT_SOFT),
        (Fit::Maybe, _) => ("may be cut", WARN, WARN_SOFT),
        (Fit::Bad, FileKind::Ir) => ("long: cut", WARN, WARN_SOFT),
        (Fit::Bad, FileKind::Amp) => ("other pedal", DANGER, DANGER_SOFT),
        (Fit::Unknown, _) => ("?", MUTED, CONTROL),
    };
    let hint = match (fit, kind) {
        (Fit::Good, FileKind::Ir) => "Short IR: fits whole in the Tank-G's 46 ms",
        (Fit::Good, FileKind::Amp) => "Amp up to 8 KB: the Tank-G's size",
        (Fit::Maybe, _) => "Medium IR: the very end may be cut",
        (Fit::Bad, FileKind::Ir) => "Long IR (reverb, room): only the first 46 ms go to the pedal",
        (Fit::Bad, FileKind::Amp) => {
            "Over 8 KB: made for another M-VAVE pedal, doesn't fit the Tank-G"
        }
        (Fit::Unknown, _) => "Unknown size",
    };
    egui::Frame::new()
        .fill(bg)
        .corner_radius(CornerRadius::same(99))
        .inner_margin(vec2(8.0, 1.0))
        .show(ui, |ui| {
            ui.label(RichText::new(text).font(semibold(11.0)).color(fg))
        })
        .response
        .on_hover_text(hint);
}
