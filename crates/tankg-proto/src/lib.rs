//! M-VAVE Tank-G protocol. Bytes only, no I/O.
//!
//! Raw frame: `00 59 | cmd | len u24 LE | body | cs`, with
//! `cs = !sum(body)`. Over USB it is packed into 7 bits (LSB first)
//! inside `F0 … F7`. See `docs/PROTOCOL.md`.

pub mod block;
pub mod pack;
pub mod patch;
pub mod preset;

pub use pack::{pack7, unpack7};
pub use patch::Patch;
pub use preset::PresetId;

use thiserror::Error;

pub const MAGIC: [u8; 2] = [0x00, 0x59];
const HEADER_LEN: usize = 6;

/// Raw frame commands.
pub mod cmd {
    pub const ACK: u8 = 0x00;
    pub const IDENTITY: u8 = 0x11;
    /// ⛔ Never send. The transport layer refuses it.
    pub const ERASE: u8 = 0x21;
    pub const WRITE: u8 = 0x22;
    pub const READ: u8 = 0x23;
}

/// Memory type used in READ/WRITE.
pub const MEM_DEV: u8 = 0x04;

/// Addresses confirmed on the Tank-G (fw `TANK-Gv2_097`).
pub mod addr {
    /// Preset bank in flash (starts with `a5 5a`).
    pub const PRESET_BANK: u32 = 0x0000_0000;
    /// Current preset (edit buffer). Read exactly [`PATCH_LEN`](super::PATCH_LEN)
    /// bytes; any other length returns a stale buffer.
    pub const CURRENT_PATCH: u32 = 0x1000_0000;
    /// Loopback (PC audio comes back through the USB input), 1 byte: 1 = on.
    /// Same as the BlackBox.
    pub const LOOPBACK: u32 = 0xD000_0000;
    /// Status: `[bank on display, active bank, letter, mode]`.
    pub const STATUS: u32 = 0x2000_0000;
    /// Select preset: write `01` to `SELECT + index`.
    /// Captured from M-EFCS on 2026-11-09 (after the save).
    pub const SELECT: u32 = 0xE000_0000;
    /// Like [`SELECT`], but plays the preset in bypass (no effects).
    /// Status shows letter `ff` meanwhile. Select again to leave.
    pub const SELECT_BYPASS: u32 = 0xE100_0000;
    /// Commit: WRITE with no data, stores in flash what was written to the bank.
    /// Captured from M-EFCS on 2026-11-09.
    pub const COMMIT: u32 = 0xF000_0000;

    /// Address of preset `p`'s record inside the bank (after `a5 5a`).
    pub fn preset_record(p: super::PresetId) -> u32 {
        PRESET_BANK + 2 + p.index() as u32 * super::PATCH_LEN as u32
    }
}

/// One preset: 21 bytes, in the bank and in the edit buffer.
pub const PATCH_LEN: usize = 21;

/// Preset bank: `a5 5a` + 36 records of [`PATCH_LEN`]
/// (same as `tankG_am4_preset.bin`).
pub const PRESET_BANK_LEN: usize = 2 + preset::COUNT as usize * PATCH_LEN;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("SysEx without F0/F7")]
    NotSysex,
    #[error("short frame ({0} bytes)")]
    Short(usize),
    #[error("bad magic: {0:02x?}")]
    BadMagic([u8; 2]),
    #[error("bad checksum: got {got:02x}, expected {want:02x}")]
    Checksum { got: u8, want: u8 },
    #[error("reply does not match the request")]
    Mismatch,
}

pub fn checksum(body: &[u8]) -> u8 {
    !body.iter().fold(0u8, |a, &b| a.wrapping_add(b))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub cmd: u8,
    pub body: Vec<u8>,
}

impl Frame {
    pub fn new(cmd: u8, body: impl Into<Vec<u8>>) -> Self {
        Self {
            cmd,
            body: body.into(),
        }
    }

    /// Raw frame (8-bit), unpacked.
    pub fn to_raw(&self) -> Vec<u8> {
        let len = self.body.len() as u32;
        let mut out = Vec::with_capacity(HEADER_LEN + self.body.len() + 1);
        out.extend_from_slice(&MAGIC);
        out.push(self.cmd);
        out.extend_from_slice(&len.to_le_bytes()[..3]);
        out.extend_from_slice(&self.body);
        out.push(checksum(&self.body));
        out
    }

    pub fn from_raw(raw: &[u8]) -> Result<Self, Error> {
        if raw.len() < HEADER_LEN + 1 {
            return Err(Error::Short(raw.len()));
        }
        if raw[..2] != MAGIC {
            return Err(Error::BadMagic([raw[0], raw[1]]));
        }
        let len = u32::from_le_bytes([raw[3], raw[4], raw[5], 0]) as usize;
        if raw.len() < HEADER_LEN + len + 1 {
            return Err(Error::Short(raw.len()));
        }
        let body = &raw[HEADER_LEN..HEADER_LEN + len];
        let got = raw[HEADER_LEN + len];
        let want = checksum(body);
        if got != want {
            return Err(Error::Checksum { got, want });
        }
        Ok(Self::new(raw[2], body))
    }

    /// SysEx message ready for the wire.
    pub fn to_sysex(&self) -> Vec<u8> {
        let mut out = vec![0xF0];
        out.extend(pack7(&self.to_raw()));
        out.push(0xF7);
        out
    }

    pub fn from_sysex(msg: &[u8]) -> Result<Self, Error> {
        match msg {
            [0xF0, inner @ .., 0xF7] => Self::from_raw(&unpack7(inner)),
            _ => Err(Error::NotSysex),
        }
    }
}

pub fn identity_query() -> Frame {
    Frame::new(cmd::IDENTITY, [])
}

/// READ/WRITE header: `type addr(u32 LE) len(u24 LE)`.
fn mem_header(addr: u32, len: usize) -> Vec<u8> {
    mem_header_of(MEM_DEV, addr, len)
}

fn mem_header_of(mem: u8, addr: u32, len: usize) -> Vec<u8> {
    let mut h = vec![mem];
    h.extend_from_slice(&addr.to_le_bytes());
    h.extend_from_slice(&(len as u32).to_le_bytes()[..3]);
    h
}

pub fn read_request(addr: u32, len: usize) -> Frame {
    Frame::new(cmd::READ, mem_header(addr, len))
}

/// WRITE: `type addr len data`. The pedal replies with an ACK (`cmd 00`).
pub fn write_request(addr: u32, data: &[u8]) -> Frame {
    write_request_mem(MEM_DEV, addr, data)
}

/// WRITE to another memory type (`05` = amp and IR blocks).
pub fn write_request_mem(mem: u8, addr: u32, data: &[u8]) -> Frame {
    let mut body = mem_header_of(mem, addr, data.len());
    body.extend_from_slice(data);
    Frame::new(cmd::WRITE, body)
}

/// `Some(true)` = ACK ok (`00`), `Some(false)` = pedal refused, `None` = not an ACK.
pub fn ack_status(resp: &Frame) -> Option<bool> {
    (resp.cmd == cmd::ACK).then(|| resp.body.first() == Some(&0))
}

/// Strips the request echo from a READ reply and returns the data.
///
/// The pedal answers reads outside the map with a stale buffer, so the
/// length is checked here; the content is the caller's to check.
pub fn parse_read_response(resp: &Frame, addr: u32, len: usize) -> Result<Vec<u8>, Error> {
    let header = mem_header(addr, len);
    if resp.cmd != cmd::READ
        || !resp.body.starts_with(&header)
        || resp.body.len() != header.len() + len
    {
        return Err(Error::Mismatch);
    }
    Ok(resp.body[header.len()..].to_vec())
}

/// Name and firmware from the reply to `0x11` (e.g. `TANK-Gv2_097`).
pub fn parse_identity(resp: &Frame) -> Result<String, Error> {
    if resp.cmd != cmd::IDENTITY {
        return Err(Error::Mismatch);
    }
    let end = resp
        .body
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(resp.body.len());
    Ok(String::from_utf8_lossy(&resp.body[..end]).into_owned())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Preset,
    Live,
}

/// Contents of [`addr::STATUS`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Status {
    /// Bank shown on the display (may blink before confirming). Hypothesis.
    pub display_bank: u8,
    /// Active bank (0–8), raw. Still valid in bypass.
    pub bank: u8,
    /// `None` when no preset was selected since boot or when the pedal is in
    /// bypass: in both cases the letter is `ff`.
    pub preset: Option<PresetId>,
    pub mode: Mode,
}

impl Status {
    pub const LEN: usize = 4;

    pub fn parse(data: &[u8]) -> Result<Self, Error> {
        let &[display_bank, bank, slot, mode] = data else {
            return Err(Error::Short(data.len()));
        };
        Ok(Self {
            display_bank,
            bank,
            preset: PresetId::from_bank_slot(bank, slot),
            mode: if mode == 0 { Mode::Live } else { Mode::Preset },
        })
    }
}

/// Program Change that selects the preset (channel 1).
pub fn program_change(p: PresetId) -> [u8; 2] {
    [0xC0, p.index()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(s: &str) -> Vec<u8> {
        s.split_whitespace()
            .map(|b| u8::from_str_radix(b, 16).unwrap())
            .collect()
    }

    #[test]
    fn identity_query_wire() {
        assert_eq!(
            identity_query().to_sysex(),
            hex("F0 00 32 45 00 00 00 40 7F 00 F7")
        );
    }

    #[test]
    fn ack_roundtrip() {
        let wire = hex("F0 00 32 01 08 00 00 00 00 7F 01 F7");
        let f = Frame::from_sysex(&wire).unwrap();
        assert_eq!(f, Frame::new(cmd::ACK, [0x00]));
        assert_eq!(f.to_sysex(), wire);
    }

    #[test]
    fn write_vector_from_ir_box() {
        let raw = hex("00 59 22 09 00 00 05 09 00 00 80 01 00 00 01 6F");
        let f = Frame::from_raw(&raw).unwrap();
        assert_eq!(f.to_raw(), raw);
        assert_eq!(
            f.to_sysex(),
            hex("F0 00 32 09 49 00 00 40 02 09 00 00 00 18 00 00 00 01 5E 01 F7")
        );
    }

    #[test]
    fn real_tank_g_identity() {
        let wire = hex(
            "F0 00 32 45 58 01 00 00 2A 41 1C 2D 6A 72 48 1D 19 5F 60 64 39 03 00 00 00 00 \
             00 00 00 00 00 00 00 00 00 00 00 00 40 2D F7",
        );
        let f = Frame::from_sysex(&wire).unwrap();
        assert_eq!(parse_identity(&f).unwrap(), "TANK-Gv2_097");
    }

    #[test]
    fn read_status_request_and_response() {
        let req = read_request(addr::STATUS, Status::LEN);
        assert_eq!(req.body, hex("04 00 00 00 20 04 00 00"));
        let mut body = req.body.clone();
        body.extend(hex("01 01 01 01"));
        let data =
            parse_read_response(&Frame::new(cmd::READ, body), addr::STATUS, Status::LEN).unwrap();
        let st = Status::parse(&data).unwrap();
        assert_eq!(st.preset, Some("2B".parse().unwrap()));
        assert_eq!(st.mode, Mode::Preset);
    }

    #[test]
    fn status_nothing_selected() {
        let st = Status::parse(&hex("00 00 ff 00")).unwrap();
        assert_eq!(st.preset, None);
        assert_eq!(st.mode, Mode::Live);
    }

    #[test]
    fn read_response_wrong_len_rejected() {
        let mut body = mem_header(addr::STATUS, 4);
        body.extend([1, 2]);
        assert_eq!(
            parse_read_response(&Frame::new(cmd::READ, body), addr::STATUS, 4),
            Err(Error::Mismatch)
        );
    }

    #[test]
    fn write_gain_frame() {
        let f = write_request(addr::CURRENT_PATCH + 6, &[0x40]);
        assert_eq!(
            f.to_raw(),
            hex("00 59 22 09 00 00 04 06 00 00 10 01 00 00 40 a4")
        );
        assert_eq!(ack_status(&Frame::new(cmd::ACK, [0x00])), Some(true));
        assert_eq!(ack_status(&Frame::new(cmd::ACK, [0x01])), Some(false));
    }

    #[test]
    fn save_frames_match_mefcs_capture() {
        // captures/mefcs-save-2026-11-09.log: "Save preset to 1A"
        let rec = hex("01 00 01 01 13 06 2f 4e 4a 43 5d 00 00 2a 1a 05 01 06 ff 00 00");
        let p1a: PresetId = "1A".parse().unwrap();
        assert_eq!(addr::preset_record(p1a), 0x0000_0002);
        assert_eq!(addr::preset_record("9D".parse().unwrap()), 2 + 35 * 21);
        let f = write_request(addr::preset_record(p1a), &rec);
        assert_eq!(f.cmd, cmd::WRITE);
        assert_eq!(f.body.len(), 8 + 21);
        assert_eq!(
            write_request(addr::COMMIT, &[]).body,
            hex("04 00 00 00 f0 00 00 00")
        );
        assert_eq!(
            write_request(addr::SELECT, &[1]).body,
            hex("04 00 00 00 e0 01 00 00 01")
        );
    }

    #[test]
    fn bad_checksum_rejected() {
        let mut raw = Frame::new(cmd::ACK, [0x00]).to_raw();
        *raw.last_mut().unwrap() ^= 1;
        assert!(matches!(Frame::from_raw(&raw), Err(Error::Checksum { .. })));
    }
}
