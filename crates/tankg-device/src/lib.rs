//! Talks to the Tank-G over the USB-MIDI port (ALSA sequencer via midir) or
//! over Bluetooth LE (BlueZ, see [`ble`]).

mod ble;

use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use tankg_proto::{self as proto, Frame, PresetId, Status, block, cmd};
use thiserror::Error;

/// The Tank-G shows up as a generic Jieli device on USB (`4c4a:c755`).
pub const PORT_USB: &str = "USB Composite Device";
const CLIENT: &str = "tankg";
const TIMEOUT: Duration = Duration::from_millis(1500);
/// On USB, bigger reads come back empty; 100 B per request works.
const READ_CHUNK_USB: usize = 100;
/// On BLE the reply comes in a single notification (MTU 517): 400 B fit.
const READ_CHUNK_BLE: usize = 400;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Tank-G not found (neither USB nor Bluetooth)")]
    NotFound,
    #[error("MIDI port busy or unavailable: {0}")]
    Port(String),
    #[error("Bluetooth: {0}")]
    Bluetooth(String),
    #[error("no reply from the pedal within {0:?}")]
    Timeout(Duration),
    #[error("command 0x{0:02x} blocked")]
    Denied(u8),
    #[error("write outside the current preset blocked (0x{addr:08x}, {len} B)")]
    WriteDenied { addr: u32, len: usize },
    #[error("the pedal rejected the write")]
    Nak,
    #[error("preset {0} does not match after saving")]
    Verify(PresetId),
    #[error("bank read does not start with a5 5a")]
    BadBank,
    #[error("backup: {0}")]
    Backup(String),
    #[error("{0}")]
    Block(String),
    #[error(transparent)]
    Proto(#[from] proto::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Finds `tankG_am4_ampCab.bin` (factory amps and IRs) from an installed
/// M-EFCS: in Tank Studio's Wine prefix or in ~/Downloads.
pub fn find_factory_bank() -> Option<std::path::PathBuf> {
    let home = std::path::PathBuf::from(std::env::var_os("HOME")?);
    let tail = "data/flutter_assets/lib/resources/bin/tankG/tankG_am4_ampCab.bin";
    [
        ".local/share/wineprefixes/mvave/drive_c/M-EFCS",
        "Downloads/M-EFCS",
    ]
    .iter()
    .map(|dir| home.join(dir).join(tail))
    .find(|p| p.is_file())
}

/// Where automatic backups go: `$XDG_DATA_HOME/tank-studio/backups`
/// (or `~/.local/share/tank-studio/backups`).
pub fn backup_dir() -> std::path::PathBuf {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".local/share"))
        })
        .unwrap_or_else(|| ".".into());
    base.join("tank-studio/backups")
}

enum Link {
    /// USB: SysEx (frame packed into 7 bits).
    Midi {
        out: MidiOutputConnection,
        _in: MidiInputConnection<()>,
    },
    /// Bluetooth: raw frame on the `ae40` service.
    Ble(ble::Ble),
}

pub struct TankG {
    link: Link,
    rx: Receiver<Frame>,
    port_name: String,
}

impl TankG {
    /// Opens over USB; failing that, over Bluetooth (only if already connected).
    pub fn open() -> Result<Self> {
        match Self::open_port(PORT_USB) {
            Err(Error::NotFound) => Self::open_bluetooth(false),
            r => r,
        }
    }

    /// Opens over Bluetooth. With `connect`, connects if it is only paired.
    pub fn open_bluetooth(connect: bool) -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let link = ble::Ble::open(tx, connect)?;
        let port_name = format!("Bluetooth {}", link.name);
        Ok(Self {
            link: Link::Ble(link),
            rx,
            port_name,
        })
    }

    pub fn is_bluetooth(&self) -> bool {
        matches!(self.link, Link::Ble(_))
    }

    /// Opens the first MIDI port whose name contains `hint` (case-insensitive).
    pub fn open_port(hint: &str) -> Result<Self> {
        let hint = hint.to_lowercase();
        let matches = |n: String| n.to_lowercase().contains(&hint);
        let port_err = |e: &dyn std::fmt::Display| Error::Port(e.to_string());

        let mut input = MidiInput::new(CLIENT).map_err(|e| port_err(&e))?;
        input.ignore(Ignore::None);
        let in_port = input
            .ports()
            .into_iter()
            .find(|p| input.port_name(p).is_ok_and(matches))
            .ok_or(Error::NotFound)?;
        let port_name = input.port_name(&in_port).unwrap_or_default();

        let output = MidiOutput::new(CLIENT).map_err(|e| port_err(&e))?;
        let out_port = output
            .ports()
            .into_iter()
            .find(|p| output.port_name(p).is_ok_and(matches))
            .ok_or(Error::NotFound)?;

        let (tx, rx) = mpsc::channel();
        let mut pending = Vec::new();
        let conn_in = input
            .connect(
                &in_port,
                "tankg-in",
                move |_, bytes, _| {
                    // ALSA may deliver a long SysEx in pieces.
                    if bytes.first() == Some(&0xF0) {
                        pending.clear();
                    }
                    pending.extend_from_slice(bytes);
                    if pending.last() == Some(&0xF7)
                        && pending.first() == Some(&0xF0)
                        && let Ok(f) = Frame::from_sysex(&std::mem::take(&mut pending))
                    {
                        let _ = tx.send(f);
                    }
                },
                (),
            )
            .map_err(|e| port_err(&e))?;
        let out = output
            .connect(&out_port, "tankg-out")
            .map_err(|e| port_err(&e))?;

        Ok(Self {
            link: Link::Midi { out, _in: conn_in },
            rx,
            port_name,
        })
    }

    pub fn port_name(&self) -> &str {
        &self.port_name
    }

    fn send(&mut self, frame: &Frame) -> Result<()> {
        match &mut self.link {
            Link::Midi { out, .. } => out
                .send(&frame.to_sysex())
                .map_err(|e| Error::Port(e.to_string())),
            Link::Ble(b) => b.send(frame),
        }
    }

    /// Sends a frame and waits for the first valid reply.
    pub fn request(&mut self, frame: &Frame) -> Result<Frame> {
        self.request_where(frame, |_| true)
    }

    /// Like [`request`](Self::request), but ignores replies that fail
    /// `accept`. Another program (the open app) may be reading at the same
    /// time, and ALSA delivers its replies to us too.
    fn request_where(&mut self, frame: &Frame, accept: impl Fn(&Frame) -> bool) -> Result<Frame> {
        // Never ERASE. WRITE only goes out through `write`, called from the
        // public functions, each with its own address.
        if frame.cmd == cmd::ERASE {
            return Err(Error::Denied(frame.cmd));
        }
        while self.rx.try_recv().is_ok() {}
        self.send(frame)?;
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let resp = self
                .rx
                .recv_timeout(left)
                .map_err(|_| Error::Timeout(TIMEOUT))?;
            if accept(&resp) {
                return Ok(resp);
            }
        }
    }

    pub fn loopback(&mut self) -> Result<bool> {
        Ok(self.read(proto::addr::LOOPBACK, 1)?[0] != 0)
    }

    pub fn set_loopback(&mut self, on: bool) -> Result<()> {
        self.write(proto::addr::LOOPBACK, &[on as u8])
    }

    /// Switches between PRESET and LIVE (last status byte), like pressing B + C.
    pub fn set_mode(&mut self, mode: proto::Mode) -> Result<()> {
        let v = match mode {
            proto::Mode::Preset => 1,
            proto::Mode::Live => 0,
        };
        self.write(proto::addr::STATUS + 3, &[v])
    }

    /// The only way out for WRITE; callers have already checked the address.
    fn write(&mut self, addr: u32, data: &[u8]) -> Result<()> {
        self.write_mem(proto::MEM_DEV, addr, data)
    }

    fn write_mem(&mut self, mem: u8, addr: u32, data: &[u8]) -> Result<()> {
        let resp = self.request_where(&proto::write_request_mem(mem, addr, data), |r| {
            proto::ack_status(r).is_some()
        })?;
        match proto::ack_status(&resp) {
            Some(true) => Ok(()),
            _ => Err(Error::Nak),
        }
    }

    /// Writes `data` to the current preset (RAM) from `offset`. Doesn't touch
    /// flash: switching presets discards it.
    pub fn write_current_patch(&mut self, offset: usize, data: &[u8]) -> Result<()> {
        let addr = proto::addr::CURRENT_PATCH + offset as u32;
        if data.is_empty() || offset + data.len() > proto::PATCH_LEN {
            return Err(Error::WriteDenied {
                addr,
                len: data.len(),
            });
        }
        self.write(addr, data)
    }

    /// Selects the preset by writing to `SELECT + index`, like M-EFCS.
    /// `bypass` plays the same preset without effects.
    pub fn select(&mut self, p: PresetId, bypass: bool) -> Result<()> {
        let base = if bypass {
            proto::addr::SELECT_BYPASS
        } else {
            proto::addr::SELECT
        };
        self.write(base + p.index() as u32, &[1])
    }

    /// Writes `rec` to preset `p` in flash and verifies by reading back. M-EFCS
    /// sequence: record in the bank, commit, select `p`.
    pub fn save_preset(&mut self, p: PresetId, rec: &[u8; proto::PATCH_LEN]) -> Result<()> {
        self.write(proto::addr::preset_record(p), rec)?;
        self.commit()?;
        self.select(p, false)?;
        self.verify(&[(p, rec)])
    }

    /// Swaps two presets in flash: two records and one commit, like M-EFCS.
    /// Selects `b` at the end.
    pub fn swap_presets(&mut self, a: PresetId, b: PresetId) -> Result<()> {
        if a == b {
            return Ok(());
        }
        let ra = self.read_record(a)?;
        let rb = self.read_record(b)?;
        self.write(proto::addr::preset_record(b), &ra)?;
        self.write(proto::addr::preset_record(a), &rb)?;
        self.commit()?;
        self.select(b, false)?;
        self.verify(&[(a, &rb), (b, &ra)])
    }

    /// Flash commit (WRITE with no data). The capture doesn't show whether an
    /// ACK comes, so it waits a bit for one and relies on the read-back after.
    fn commit(&mut self) -> Result<()> {
        match self.write(proto::addr::COMMIT, &[]) {
            Ok(()) | Err(Error::Timeout(_)) => {}
            Err(e) => return Err(e),
        }
        // Writing flash takes a while; commands in between may cancel it.
        std::thread::sleep(Duration::from_millis(300));
        Ok(())
    }

    fn read_record(&mut self, p: PresetId) -> Result<[u8; proto::PATCH_LEN]> {
        let v = self.read(proto::addr::preset_record(p), proto::PATCH_LEN)?;
        Ok(v.try_into().expect("read returns the requested length"))
    }

    fn verify(&mut self, want: &[(PresetId, &[u8; proto::PATCH_LEN])]) -> Result<()> {
        for (p, rec) in want {
            if &self.read_record(*p)? != *rec {
                return Err(Error::Verify(*p));
            }
        }
        Ok(())
    }

    /// Reads the bank and writes it to `dir/tankg-presets-<unix>.bin`. Runs
    /// before every flash write. Returns the path.
    pub fn backup_bank(&mut self, dir: &std::path::Path) -> Result<std::path::PathBuf> {
        let bank = self.read_preset_bank()?;
        if bank.get(..2) != Some(&[0xA5, 0x5A]) {
            return Err(Error::BadBank);
        }
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let path = dir.join(format!("tankg-presets-{secs}.bin"));
        std::fs::create_dir_all(dir)
            .and_then(|_| std::fs::write(&path, &bank))
            .map_err(|e| Error::Backup(format!("{}: {e}", path.display())))?;
        Ok(path)
    }

    /// Sends an amp or IR record and plays it (audition), without saving.
    /// `progress(sent, total)` after each chunk.
    pub fn audition_block(
        &mut self,
        kind: block::Kind,
        rec: &[u8],
        mut progress: impl FnMut(usize, usize),
    ) -> Result<()> {
        block::check_record(kind, rec).map_err(Error::Block)?;
        for (i, chunk) in rec.chunks(block::CHUNK).enumerate() {
            self.write_mem(block::MEM_BLOCK, (i * block::CHUNK) as u32, chunk)?;
            progress(((i + 1) * block::CHUNK).min(rec.len()), rec.len());
        }
        self.write_mem(block::MEM_BLOCK, block::AUDITION, &[kind.audition_value()])
    }

    /// Level of the block being auditioned (record byte 19).
    pub fn set_block_level(&mut self, kind: block::Kind, level: u8) -> Result<()> {
        self.write_mem(block::MEM_BLOCK, kind.level_addr(), &[level])
    }

    /// Saves the auditioned block to slot (1…). Only after [`Self::audition_block`].
    pub fn save_block(&mut self, kind: block::Kind, slot: u8) -> Result<()> {
        if !(1..=kind.slots()).contains(&slot) {
            return Err(Error::Block(format!(
                "invalid {} slot: {slot}",
                kind.name()
            )));
        }
        self.write_mem(block::MEM_BLOCK, kind.save_addr(), &[slot - 1])?;
        // Writing flash takes a while; commands in between may cancel it.
        std::thread::sleep(Duration::from_millis(1500));
        Ok(())
    }

    pub fn identify(&mut self) -> Result<String> {
        let resp = self.request(&proto::identity_query())?;
        Ok(proto::parse_identity(&resp)?)
    }

    pub fn read(&mut self, addr: u32, len: usize) -> Result<Vec<u8>> {
        let mut data = Vec::with_capacity(len);
        while data.len() < len {
            let off = data.len();
            let chunk = if self.is_bluetooth() {
                READ_CHUNK_BLE
            } else {
                READ_CHUNK_USB
            };
            let n = chunk.min(len - off);
            let a = addr + off as u32;
            let resp = self.request_where(&proto::read_request(a, n), |r| {
                proto::parse_read_response(r, a, n).is_ok()
            })?;
            data.extend(proto::parse_read_response(&resp, a, n)?);
        }
        Ok(data)
    }

    pub fn status(&mut self) -> Result<Status> {
        let data = self.read(proto::addr::STATUS, Status::LEN)?;
        Ok(Status::parse(&data)?)
    }

    /// USB: Program Change. Bluetooth: a Program Change would go through the
    /// BLE-MIDI port, so it writes to `SELECT` like M-EFCS.
    pub fn select_preset(&mut self, p: PresetId) -> Result<()> {
        match &mut self.link {
            Link::Midi { out, .. } => out
                .send(&proto::program_change(p))
                .map_err(|e| Error::Port(e.to_string())),
            Link::Ble(_) => self.select(p, false),
        }
    }

    /// Current preset (edit buffer, RAM), one 21-byte record.
    pub fn read_current_patch(&mut self) -> Result<Vec<u8>> {
        self.read(proto::addr::CURRENT_PATCH, proto::PATCH_LEN)
    }

    pub fn read_preset_bank(&mut self) -> Result<Vec<u8>> {
        self.read(proto::addr::PRESET_BANK, proto::PRESET_BANK_LEN)
    }
}
