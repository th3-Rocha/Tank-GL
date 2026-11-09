//! Amp and IR (CAB) blocks: memory type `05`.
//!
//! Captured from M-EFCS on 2026-11-09 (`docs/PROTOCOL.md`): the record goes in
//! [`CHUNK`]-sized pieces from address 0, then *audition* (plays without
//! saving) and, to keep it, *save* to the slot.

/// Memory type of the blocks (presets use `04`).
pub const MEM_BLOCK: u8 = 0x05;
/// Size of the chunks M-EFCS sends.
pub const CHUNK: usize = 1015;
/// Plays the block just sent without saving: `01` = amp, `02` = IR.
pub const AUDITION: u32 = 0xA000_0000;
/// Saves the auditioned block to the slot: write `slot − 1` to `SAVE + 0x100·type`.
pub const SAVE: u32 = 0xF000_0000;

/// IR: `CAB\0` + name (14 B) + format + level + samples.
pub const CAB_LEN: usize = 20 + CAB_SAMPLES * 3;
/// 2048 int24 LE samples at 44.1 kHz.
pub const CAB_SAMPLES: usize = 2048;
pub const CAB_RATE: u32 = 44_100;
/// Amp: `AM4\0` + name (14 B) + byte 18 + level + gain (2 B) + neural network.
/// Factory ones fill the whole slot; community ones vary (7868 B, for
/// example), so this is the maximum.
pub const AMP_LEN: usize = 8192;
/// Smallest amp accepted: the header and some network.
const AMP_MIN: usize = 1024;
/// Level M-EFCS uses when importing an IR.
pub const CAB_DEFAULT_LEVEL: u8 = 0x32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Amp,
    Cab,
}

impl Kind {
    pub fn magic(self) -> &'static [u8; 4] {
        match self {
            Kind::Amp => b"AM4\0",
            Kind::Cab => b"CAB\0",
        }
    }

    pub fn record_len(self) -> usize {
        match self {
            Kind::Amp => AMP_LEN,
            Kind::Cab => CAB_LEN,
        }
    }

    /// Slots on the pedal (numbered from 1).
    pub fn slots(self) -> u8 {
        match self {
            Kind::Amp => 9,
            Kind::Cab => 8,
        }
    }

    pub fn audition_value(self) -> u8 {
        match self {
            Kind::Amp => 1,
            Kind::Cab => 2,
        }
    }

    /// Address of the auditioned block: amp = family 1, IR = family 2.
    fn live_base(self) -> u32 {
        match self {
            Kind::Amp => 0x1000_0000,
            Kind::Cab => 0x2000_0000,
        }
    }

    /// Level (record byte 19) of the auditioned block.
    pub fn level_addr(self) -> u32 {
        self.live_base() + 0x13
    }

    pub fn save_addr(self) -> u32 {
        match self {
            Kind::Amp => SAVE,
            Kind::Cab => SAVE + 0x100,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Kind::Amp => "amp",
            Kind::Cab => "IR",
        }
    }
}

/// Checks size and signature before sending anything.
pub fn check_record(kind: Kind, rec: &[u8]) -> Result<(), String> {
    let ok = match kind {
        Kind::Cab => rec.len() == CAB_LEN,
        Kind::Amp => (AMP_MIN..=AMP_LEN).contains(&rec.len()),
    };
    if !ok {
        return Err(match kind {
            Kind::Cab => format!("IR has {} bytes; the pedal expects {CAB_LEN}", rec.len()),
            Kind::Amp => format!(
                "amp has {} bytes; the Tank-G takes up to {AMP_LEN} (this one is probably for another pedal)",
                rec.len()
            ),
        });
    }
    if &rec[..4] != kind.magic() {
        let got = String::from_utf8_lossy(&rec[..4]);
        return Err(format!(
            "{} without the {:?} signature (got {got:?})",
            kind.name(),
            String::from_utf8_lossy(kind.magic())
        ));
    }
    Ok(())
}

/// Builds the IR record from int24 samples (already at 44.1 kHz).
/// The name is left empty and the format is `01`, like M-EFCS does.
pub fn cab_record(samples: &[i32], level: u8) -> Vec<u8> {
    let mut rec = Vec::with_capacity(CAB_LEN);
    rec.extend_from_slice(b"CAB\0");
    rec.extend_from_slice(&[0; 14]);
    rec.push(0x01);
    rec.push(level);
    for i in 0..CAB_SAMPLES {
        let v = samples
            .get(i)
            .copied()
            .unwrap_or(0)
            .clamp(-0x80_0000, 0x7F_FFFF);
        rec.extend_from_slice(&v.to_le_bytes()[..3]);
    }
    rec
}

/// What the WAV contained, to show on screen.
#[derive(Debug, Clone, PartialEq)]
pub struct WavInfo {
    pub rate: u32,
    pub channels: u16,
    pub bits: u16,
    pub float: bool,
    pub frames: usize,
}

impl WavInfo {
    pub fn describe(&self) -> String {
        let ch = if self.channels == 1 {
            "mono".to_owned()
        } else {
            format!("{} channels (uses the 1st)", self.channels)
        };
        let fmt = if self.float {
            format!("{}-bit float", self.bits)
        } else {
            format!("{}-bit", self.bits)
        };
        let ms = self.frames as f64 * 1000.0 / self.rate as f64;
        format!(
            "{:.1} kHz, {ch}, {fmt}, {ms:.0} ms",
            self.rate as f64 / 1000.0
        )
    }
}

/// Reads a WAV (PCM 16/24/32-bit or float 32/64) and returns 2048 int24 samples
/// at 44.1 kHz: 1st channel, resampled if needed, cut or padded with zeros.
/// Doesn't normalize: the file's volume goes as is, like M-EFCS.
pub fn wav_to_cab(bytes: &[u8]) -> Result<(Vec<i32>, WavInfo), String> {
    if bytes.len() < 12 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("not a WAV file".into());
    }
    let mut pos = 12;
    let mut fmt: Option<(u16, u16, u32, u16)> = None;
    let mut data: Option<&[u8]> = None;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let size = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap()) as usize;
        let body = &bytes[pos + 8..(pos + 8 + size).min(bytes.len())];
        match id {
            b"fmt " if body.len() >= 16 => {
                let mut tag = u16::from_le_bytes([body[0], body[1]]);
                let channels = u16::from_le_bytes([body[2], body[3]]);
                let rate = u32::from_le_bytes(body[4..8].try_into().unwrap());
                let bits = u16::from_le_bytes([body[14], body[15]]);
                // WAVE_FORMAT_EXTENSIBLE: the real type is in the subformat.
                if tag == 0xFFFE && body.len() >= 26 {
                    tag = u16::from_le_bytes([body[24], body[25]]);
                }
                fmt = Some((tag, channels, rate, bits));
            }
            b"data" => data = Some(body),
            _ => {}
        }
        pos += 8 + size + (size & 1);
    }
    let (tag, channels, rate, bits) = fmt.ok_or("WAV without a fmt chunk")?;
    let data = data.ok_or("WAV without a data chunk")?;
    let float = match (tag, bits) {
        (1, 16 | 24 | 32) => false,
        (3, 32 | 64) => true,
        _ => return Err(format!("unsupported WAV format (type {tag}, {bits} bits)")),
    };
    if channels == 0 || rate == 0 {
        return Err("WAV with an invalid header".into());
    }
    let width = bits as usize / 8;
    let frame = width * channels as usize;
    let frames = data.len() / frame;
    // 1st channel, scaled to int24
    let first: Vec<f64> = (0..frames)
        .map(|i| {
            let b = &data[i * frame..i * frame + width];
            match (float, bits) {
                (false, 16) => i16::from_le_bytes([b[0], b[1]]) as f64 * 256.0,
                (false, 24) => (i32::from_le_bytes([0, b[0], b[1], b[2]]) >> 8) as f64,
                (false, 32) => i32::from_le_bytes(b.try_into().unwrap()) as f64 / 256.0,
                (true, 32) => f32::from_le_bytes(b.try_into().unwrap()) as f64 * 8_388_607.0,
                (true, 64) => f64::from_le_bytes(b.try_into().unwrap()) * 8_388_607.0,
                _ => unreachable!(),
            }
        })
        .collect();
    let samples = (0..CAB_SAMPLES)
        .map(|i| {
            let v = if rate == CAB_RATE {
                first.get(i).copied().unwrap_or(0.0)
            } else {
                // linear resampling; a cab IR is short and this is enough
                let t = i as f64 * rate as f64 / CAB_RATE as f64;
                let k = t.floor() as usize;
                let f = t - k as f64;
                let a = first.get(k).copied().unwrap_or(0.0);
                let b = first.get(k + 1).copied().unwrap_or(0.0);
                a + (b - a) * f
            };
            v.round().clamp(-8_388_608.0, 8_388_607.0) as i32
        })
        .collect();
    Ok((
        samples,
        WavInfo {
            rate,
            channels,
            bits,
            float,
            frames,
        },
    ))
}

/// Factory record of slot (1…) inside `tankG_am4_ampCab.bin`, which ships
/// with M-EFCS: 9 amps of 16 KiB followed by 8 IRs of 8 KiB. For an IR only
/// the first [`CAB_LEN`] bytes count; the rest of the slot is padding.
pub fn factory_record(bank: &[u8], kind: Kind, slot: u8) -> Option<Vec<u8>> {
    if !(1..=kind.slots()).contains(&slot) {
        return None;
    }
    let i = (slot - 1) as usize;
    let start = match kind {
        Kind::Amp => i * 0x4000,
        Kind::Cab => 0x2_4000 + i * 0x2000,
    };
    let rec = bank.get(start..start + kind.record_len())?.to_vec();
    check_record(kind, &rec).ok()?;
    Some(rec)
}

/// Name stored in the record (bytes 4–17), without the leading number the factory uses.
pub fn record_name(rec: &[u8]) -> String {
    let raw = rec.get(4..18).unwrap_or_default();
    let s: String = raw
        .iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as char)
        .collect();
    // The factory puts the slot number in front ("1J120_CL"); strip it only if a name remains.
    let rest = s.trim_start_matches(|c: char| c.is_ascii_digit());
    let name = if rest.is_empty() { s.as_str() } else { rest };
    name.replace('_', " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wav(rate: u32, channels: u16, bits: u16, tag: u16, data: &[u8]) -> Vec<u8> {
        let mut v = b"RIFF\0\0\0\0WAVEfmt ".to_vec();
        v.extend(16u32.to_le_bytes());
        v.extend(tag.to_le_bytes());
        v.extend(channels.to_le_bytes());
        v.extend(rate.to_le_bytes());
        v.extend((rate * channels as u32 * bits as u32 / 8).to_le_bytes());
        v.extend((channels * bits / 8).to_le_bytes());
        v.extend(bits.to_le_bytes());
        v.extend(b"data");
        v.extend((data.len() as u32).to_le_bytes());
        v.extend(data);
        v
    }

    /// Same file as the capture (`captures/ir-teste-impulso.wav`): the markers
    /// land where M-EFCS sent them.
    #[test]
    fn impulse_matches_mefcs_capture() {
        let mut s = vec![0i32; 2048];
        s[0] = 0x40_0000;
        s[100] = -0x10_0000;
        s[1000] = 0x01_2345;
        let data: Vec<u8> = s
            .iter()
            .flat_map(|v| v.to_le_bytes()[..3].to_vec())
            .collect();
        let (got, info) = wav_to_cab(&wav(44_100, 1, 24, 1, &data)).unwrap();
        assert_eq!(info.frames, 2048);
        let rec = cab_record(&got, CAB_DEFAULT_LEVEL);
        assert_eq!(rec.len(), 6164);
        assert_eq!(
            &rec[..20],
            &[
                b'C', b'A', b'B', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01, 0x32
            ]
        );
        assert_eq!(&rec[20..23], &[0x00, 0x00, 0x40]);
        assert_eq!(&rec[320..323], &(-0x10_0000i32).to_le_bytes()[..3]);
        assert_eq!(&rec[3020..3023], &[0x45, 0x23, 0x01]);
        check_record(Kind::Cab, &rec).unwrap();
    }

    #[test]
    fn stereo_16bit_48k_is_converted() {
        // impulse on the left channel, junk on the right
        let mut data = Vec::new();
        for i in 0..4800 {
            let l: i16 = if i == 0 { 16384 } else { 0 };
            data.extend(l.to_le_bytes());
            data.extend(1000i16.to_le_bytes());
        }
        let (got, info) = wav_to_cab(&wav(48_000, 2, 16, 1, &data)).unwrap();
        assert_eq!((info.rate, info.channels), (48_000, 2));
        assert_eq!(got.len(), 2048);
        assert_eq!(got[0], 16384 * 256);
        assert!(
            got[10..].iter().all(|&v| v == 0),
            "the right channel must not leak"
        );
    }

    #[test]
    fn rejects_bad_records() {
        assert!(wav_to_cab(b"not a wav").is_err());
        assert!(check_record(Kind::Amp, &[0; 10]).is_err());
        let mut amp = vec![0u8; AMP_LEN];
        amp[..4].copy_from_slice(b"AM4\0");
        check_record(Kind::Amp, &amp).unwrap();
        check_record(Kind::Amp, &amp[..7868]).unwrap();
        assert!(
            check_record(Kind::Amp, &[amp.clone(), vec![0; 6000]].concat()).is_err(),
            "14 KB is for another pedal"
        );
        amp[2] = b'2';
        assert!(check_record(Kind::Amp, &amp).is_err());
    }

    #[test]
    fn record_names() {
        let mut r = vec![0u8; 20];
        r[4..12].copy_from_slice(b"1J120_CL");
        assert_eq!(record_name(&r), "J120 CL");
        r[4..12].copy_from_slice(b"188222\0\0");
        assert_eq!(record_name(&r), "188222");
    }

    #[test]
    fn addresses_match_capture() {
        assert_eq!(Kind::Cab.level_addr(), 0x2000_0013);
        assert_eq!(Kind::Amp.level_addr(), 0x1000_0013);
        assert_eq!(Kind::Cab.save_addr(), 0xF000_0100);
        assert_eq!(Kind::Amp.save_addr(), 0xF000_0000);
        assert_eq!(
            (Kind::Amp.audition_value(), Kind::Cab.audition_value()),
            (1, 2)
        );
        assert_eq!(CAB_LEN.div_ceil(CHUNK), 7);
        assert_eq!(AMP_LEN.div_ceil(CHUNK), 9);
    }
}
