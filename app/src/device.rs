//! Thread that owns the pedal connection. The UI sends `Cmd` and gets `Event`.
//!
//! The pedal doesn't report footswitch presses, so the thread reads the
//! status every `POLL`. Any transport error drops the connection and the next
//! round tries to reopen it.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};

use tankg_device::TankG;
use tankg_proto::{Mode, Patch, PresetId, Status, block};

const POLL: Duration = Duration::from_millis(600);
/// The pedal takes a moment to update its status after a command.
const SETTLE: Duration = Duration::from_millis(150);

pub enum Cmd {
    Select(PresetId),
    /// One byte of the current preset (RAM, not saved).
    SetByte {
        offset: usize,
        value: u8,
    },
    SetMode(Mode),
    SetLoopback(bool),
    Backup(PathBuf),
    /// Writes `rec` to preset `target` in flash (backup before, read back after).
    Save {
        target: PresetId,
        rec: [u8; tankg_proto::PATCH_LEN],
    },
    /// Swaps two presets in flash.
    Swap(PresetId, PresetId),
    /// Bypass of the current preset.
    Bypass(bool),
    /// Sends an amp/IR record and plays it (without saving). With `slot`, saves it after.
    Block {
        kind: block::Kind,
        rec: Vec<u8>,
        slot: Option<u8>,
    },
    /// Level of the block being auditioned.
    BlockLevel(block::Kind, u8),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Ok,
    Danger,
}

pub enum Event {
    Connected {
        firmware: String,
        bluetooth: bool,
        bank: Option<Vec<Patch>>,
    },
    State {
        status: Status,
        patch: Patch,
        loopback: bool,
    },
    Disconnected(String),
    /// Bank read back after a save or swap.
    Bank(Vec<Patch>),
    /// Amp/IR upload: bytes sent, total. `None` = done.
    BlockProgress(Option<(usize, usize)>),
    /// Line for the MIDI monitor. `out` = from the PC to the pedal.
    Log {
        out: bool,
        bytes: String,
        text: String,
    },
    Notice(Tone, String),
}

pub struct Device {
    tx: Sender<Cmd>,
    pub rx: Receiver<Event>,
}

impl Device {
    pub fn spawn(ctx: egui::Context) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (ev_tx, ev_rx) = mpsc::channel();
        thread::Builder::new()
            .name("tankg".into())
            .spawn(move || {
                Worker {
                    cmds: cmd_rx,
                    events: ev_tx,
                    ctx,
                    dev: None,
                    last_error: None,
                    last_preset: None,
                }
                .run()
            })
            .expect("pedal thread");
        Self {
            tx: cmd_tx,
            rx: ev_rx,
        }
    }

    pub fn send(&self, cmd: Cmd) {
        let _ = self.tx.send(cmd);
    }
}

struct Worker {
    cmds: Receiver<Cmd>,
    events: Sender<Event>,
    ctx: egui::Context,
    dev: Option<TankG>,
    last_error: Option<String>,
    /// Last preset seen. In bypass the pedal doesn't say which.
    last_preset: Option<PresetId>,
}

impl Worker {
    fn emit(&self, ev: Event) {
        let _ = self.events.send(ev);
        self.ctx.request_repaint();
    }

    fn log(&self, out: bool, bytes: impl Into<String>, text: impl Into<String>) {
        self.emit(Event::Log {
            out,
            bytes: bytes.into(),
            text: text.into(),
        });
    }

    fn run(mut self) {
        let mut next_poll = Instant::now();
        loop {
            if self.dev.is_none() {
                self.connect();
            }
            let wait = next_poll.saturating_duration_since(Instant::now());
            match self.cmds.recv_timeout(wait) {
                Ok(first) => {
                    let mut batch = vec![first];
                    batch.extend(self.cmds.try_iter());
                    for cmd in coalesce(batch) {
                        self.handle(cmd);
                    }
                }
                Err(RecvTimeoutError::Timeout) => {
                    self.poll();
                    next_poll = Instant::now() + POLL;
                }
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
    }

    fn connect(&mut self) {
        let opened = TankG::open().and_then(|mut d| {
            let firmware = d.identify()?;
            Ok((d, firmware))
        });
        match opened {
            Ok((mut d, firmware)) => {
                self.log(false, "IDENTITY 0x11", format!("Connected: {firmware}"));
                let bank = d.read_preset_bank().ok().and_then(|b| Patch::from_bank(&b));
                if bank.is_some() {
                    self.log(false, "READ 0x00000000", "Preset bank read (36 × 21 B)");
                }
                let bluetooth = d.is_bluetooth();
                self.dev = Some(d);
                self.last_error = None;
                self.emit(Event::Connected {
                    firmware,
                    bluetooth,
                    bank,
                });
            }
            Err(e) => self.disconnected(e.to_string()),
        }
    }

    fn disconnected(&mut self, err: String) {
        self.dev = None;
        // Only report when the reason changes, to avoid needless repaints.
        if self.last_error.as_deref() != Some(&err) {
            self.last_error = Some(err.clone());
            self.emit(Event::Disconnected(err));
        }
    }

    /// Runs `f` on the pedal; if the transport fails, drops the connection.
    fn with<T>(&mut self, f: impl FnOnce(&mut TankG) -> tankg_device::Result<T>) -> Option<T> {
        let d = self.dev.as_mut()?;
        match f(d) {
            Ok(v) => Some(v),
            Err(e) => {
                self.disconnected(e.to_string());
                None
            }
        }
    }

    fn poll(&mut self) {
        let read = self.with(|d| {
            let status = d.status()?;
            let raw = d.read_current_patch()?;
            let loopback = d.loopback()?;
            Ok((status, raw, loopback))
        });
        if let Some((status, raw, loopback)) = read {
            match Patch::parse(&raw) {
                Some(patch) => {
                    if status.preset.is_some() {
                        self.last_preset = status.preset;
                    }
                    self.emit(Event::State {
                        status,
                        patch,
                        loopback,
                    })
                }
                None => self.disconnected("current preset has the wrong size".into()),
            }
        }
    }

    fn handle(&mut self, cmd: Cmd) {
        if self.dev.is_none() {
            self.emit(Event::Notice(
                Tone::Danger,
                "The pedal is not connected.".into(),
            ));
            return;
        }
        match cmd {
            Cmd::Select(p) => {
                let [s, n] = tankg_proto::program_change(p);
                self.log(
                    true,
                    format!("C{:X} {n:02X}", s & 0x0F),
                    format!("Program Change → {p}"),
                );
                if self.with(|d| d.select_preset(p)).is_some() {
                    thread::sleep(SETTLE);
                    self.poll();
                }
            }
            Cmd::SetByte { offset, value } => {
                let addr = tankg_proto::addr::CURRENT_PATCH + offset as u32;
                if self
                    .with(|d| d.write_current_patch(offset, &[value]))
                    .is_some()
                {
                    self.log(
                        true,
                        format!("WRITE 0x{addr:08x} = {value:02x}"),
                        format!("byte {offset} → {value}"),
                    );
                } else {
                    self.emit(Event::Notice(
                        Tone::Danger,
                        "The pedal did not accept the change.".into(),
                    ));
                }
            }
            Cmd::SetMode(m) => {
                let (v, name) = if m == Mode::Preset {
                    (1, "PRESET")
                } else {
                    (0, "LIVE")
                };
                self.log(
                    true,
                    format!("WRITE 0x20000003 = 0{v}"),
                    format!("Mode {name}"),
                );
                if self.with(|d| d.set_mode(m)).is_some() {
                    thread::sleep(SETTLE);
                    self.poll();
                }
            }
            Cmd::SetLoopback(on) => {
                self.log(
                    true,
                    format!("WRITE 0xd0000000 = 0{}", on as u8),
                    format!("Loopback {}", if on { "on" } else { "off" }),
                );
                if self.with(|d| d.set_loopback(on)).is_some() {
                    self.poll();
                }
            }
            Cmd::Save { target, rec } => {
                let Some(bk) = self.backup() else { return };
                let addr = tankg_proto::addr::preset_record(target);
                self.log(
                    true,
                    format!("WRITE 0x{addr:08x} (21 B) + COMMIT 0xf0000000"),
                    format!("Save to {target}"),
                );
                if self.with(|d| d.save_preset(target, &rec)).is_some() {
                    self.after_flash(format!("Saved to {target} (verified by reading the pedal back). Backup of the previous bank: {}", bk.display()));
                }
            }
            Cmd::Swap(a, b) => {
                let Some(bk) = self.backup() else { return };
                self.log(
                    true,
                    "WRITE 2 records + COMMIT 0xf0000000",
                    format!("Swap {a} ↔ {b}"),
                );
                if self.with(|d| d.swap_presets(a, b)).is_some() {
                    self.after_flash(format!("{a} and {b} swapped (verified by reading the pedal back). Backup of the previous bank: {}", bk.display()));
                }
            }
            Cmd::Bypass(on) => {
                let Some(p) = self.last_preset else {
                    self.emit(Event::Notice(
                        Tone::Danger,
                        "Pick a preset before using Bypass.".into(),
                    ));
                    return;
                };
                let base = if on { 0xE100_0000u32 } else { 0xE000_0000 };
                self.log(
                    true,
                    format!("WRITE 0x{:08x} = 01", base + p.index() as u32),
                    format!("Bypass {} ({p})", if on { "on" } else { "off" }),
                );
                if self.with(|d| d.select(p, on)).is_some() {
                    thread::sleep(SETTLE);
                    self.poll();
                }
            }
            Cmd::Block { kind, rec, slot } => {
                let what = format!("{} {:?}", kind.name(), block::record_name(&rec));
                self.log(
                    true,
                    format!(
                        "WRITE type 05, {} B in chunks of {}",
                        rec.len(),
                        block::CHUNK
                    ),
                    format!("Send {what}"),
                );
                let events = self.events.clone();
                let ctx = self.ctx.clone();
                let sent = self.with(|d| {
                    d.audition_block(kind, &rec, |done, total| {
                        let _ = events.send(Event::BlockProgress(Some((done, total))));
                        ctx.request_repaint();
                    })
                });
                self.emit(Event::BlockProgress(None));
                if sent.is_none() {
                    self.emit(Event::Notice(
                        Tone::Danger,
                        format!("Sending the {} failed.", kind.name()),
                    ));
                    return;
                }
                self.log(
                    true,
                    format!(
                        "WRITE type 05 0x{:08x} = {:02x}",
                        block::AUDITION,
                        kind.audition_value()
                    ),
                    "Audition",
                );
                let Some(slot) = slot else {
                    self.emit(Event::Notice(Tone::Ok, format!("{what} playing on the pedal (not saved). Switching presets goes back to what is saved.")));
                    return;
                };
                self.log(
                    true,
                    format!(
                        "WRITE type 05 0x{:08x} = {:02x}",
                        kind.save_addr(),
                        slot - 1
                    ),
                    format!("Save to slot {slot}"),
                );
                if self.with(|d| d.save_block(kind, slot)).is_some() {
                    self.emit(Event::Notice(
                        Tone::Ok,
                        format!("{what} saved to {} slot {slot}.", kind.name()),
                    ));
                    self.poll();
                }
            }
            Cmd::BlockLevel(kind, level) => {
                if self.with(|d| d.set_block_level(kind, level)).is_some() {
                    self.log(
                        true,
                        format!("WRITE type 05 0x{:08x} = {level:02x}", kind.level_addr()),
                        format!("{} level → {level}", kind.name()),
                    );
                }
            }
            Cmd::Backup(path) => {
                let Some(bank) = self.with(|d| d.read_preset_bank()) else {
                    return;
                };
                let msg = if bank.get(..2) != Some(&[0xA5, 0x5A]) {
                    (
                        Tone::Danger,
                        format!(
                            "Backup failed: the bank read doesn't start with a5 5a ({:02x?})",
                            &bank[..bank.len().min(8)]
                        ),
                    )
                } else {
                    match std::fs::write(&path, &bank) {
                        Ok(()) => (
                            Tone::Ok,
                            format!("Backup saved: {} bytes to {}", bank.len(), path.display()),
                        ),
                        Err(e) => (
                            Tone::Danger,
                            format!("Backup failed writing {}: {e}", path.display()),
                        ),
                    }
                };
                self.emit(Event::Notice(msg.0, msg.1));
            }
        }
    }
}

impl Worker {
    /// Automatic backup before writing to flash. `None` = don't write.
    fn backup(&mut self) -> Option<PathBuf> {
        let dir = tankg_device::backup_dir();
        let r = self.dev.as_mut()?.backup_bank(&dir);
        match r {
            Ok(p) => Some(p),
            Err(e) => {
                self.emit(Event::Notice(
                    Tone::Danger,
                    format!("Nothing was written: the backup before saving failed ({e})."),
                ));
                None
            }
        }
    }

    /// After writing to flash: reads the bank and status again.
    fn after_flash(&mut self, msg: String) {
        if let Some(bank) = self
            .with(|d| d.read_preset_bank())
            .and_then(|b| Patch::from_bank(&b))
        {
            self.emit(Event::Bank(bank));
        }
        self.poll();
        self.emit(Event::Notice(Tone::Ok, msg));
    }
}

/// A drag sends dozens of writes to the same byte: only the last one matters.
fn coalesce(batch: Vec<Cmd>) -> Vec<Cmd> {
    let mut out: Vec<Cmd> = Vec::with_capacity(batch.len());
    for cmd in batch {
        if let Cmd::SetByte { offset, .. } = cmd {
            out.retain(|c| !matches!(c, Cmd::SetByte { offset: o, .. } if *o == offset));
        }
        if let Cmd::BlockLevel(kind, _) = cmd {
            out.retain(|c| !matches!(c, Cmd::BlockLevel(k, _) if *k == kind));
        }
        out.push(cmd);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_last_write_per_byte() {
        let out = coalesce(vec![
            Cmd::SetByte {
                offset: 6,
                value: 1,
            },
            Cmd::SetByte {
                offset: 7,
                value: 9,
            },
            Cmd::SetByte {
                offset: 6,
                value: 3,
            },
        ]);
        let got: Vec<_> = out
            .iter()
            .map(|c| match c {
                Cmd::SetByte { offset, value } => (*offset, *value),
                _ => unreachable!(),
            })
            .collect();
        assert_eq!(got, [(7, 9), (6, 3)]);
    }
}
