//! A Tank-G preset: 21 bytes, the same in the bank and the edit buffer.
//!
//! Mapped by turning each knob in LIVE mode and reading `0x1000_0000`
//! (`captures/knobs-4C-2026-11-04.log`). Knobs follow the panel's order.

use crate::PATCH_LEN;

/// Continuous knobs go from 0 to 127.
pub const KNOB_MAX: u8 = 127;
/// AMP TYPE: 0–8 (9 amps).
pub const AMP_TYPES: u8 = 9;
/// IR CAB: 0 = OFF, 1–8 = slots.
pub const IR_SLOTS: u8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Patch {
    /// Footswitch D in LIVE mode.
    pub amp_on: bool,
    /// Footswitch C.
    pub mod_on: bool,
    /// Footswitch B.
    pub delay_on: bool,
    /// Footswitch A.
    pub reverb_on: bool,
    pub noise_gate: u8,
    pub amp_type: u8,
    pub gain: u8,
    pub treble: u8,
    pub middle: u8,
    pub bass: u8,
    pub volume: u8,
    /// Knob position; the zone picks Chorus/Phaser/Tremolo.
    pub mod_fx: u8,
    pub mod_speed: u8,
    /// Knob position; the zone picks Analog/Tape/Dual.
    pub delay_mix: u8,
    pub delay_time: u8,
    /// Knob position; the zone picks Room/Spring/Cloud.
    pub reverb_decay: u8,
    pub reverb_mix: u8,
    pub ir_cab: u8,
    /// Bytes 18–20, meaning still unknown. Factory: `ff`/`00`, `00`, `00`.
    /// Byte 18 may be the LED color. Kept so nothing is lost on the way back.
    pub unknown: [u8; 3],
}

impl Patch {
    pub fn parse(b: &[u8]) -> Option<Self> {
        let b: &[u8; PATCH_LEN] = b.try_into().ok()?;
        Some(Self {
            amp_on: b[0] != 0,
            mod_on: b[1] != 0,
            delay_on: b[2] != 0,
            reverb_on: b[3] != 0,
            noise_gate: b[4],
            amp_type: b[5],
            gain: b[6],
            treble: b[7],
            middle: b[8],
            bass: b[9],
            volume: b[10],
            mod_fx: b[11],
            mod_speed: b[12],
            delay_mix: b[13],
            delay_time: b[14],
            reverb_decay: b[15],
            reverb_mix: b[16],
            ir_cab: b[17],
            unknown: [b[18], b[19], b[20]],
        })
    }

    pub fn to_bytes(&self) -> [u8; PATCH_LEN] {
        let [u0, u1, u2] = self.unknown;
        [
            self.amp_on as u8,
            self.mod_on as u8,
            self.delay_on as u8,
            self.reverb_on as u8,
            self.noise_gate,
            self.amp_type,
            self.gain,
            self.treble,
            self.middle,
            self.bass,
            self.volume,
            self.mod_fx,
            self.mod_speed,
            self.delay_mix,
            self.delay_time,
            self.reverb_decay,
            self.reverb_mix,
            self.ir_cab,
            u0,
            u1,
            u2,
        ]
    }

    /// The 36 presets of a bank read from `0x0000_0000`.
    pub fn from_bank(bank: &[u8]) -> Option<Vec<Self>> {
        let body = bank.strip_prefix(&[0xA5, 0x5A])?;
        if body.len() != crate::preset::COUNT as usize * PATCH_LEN {
            return None;
        }
        body.chunks(PATCH_LEN).map(Self::parse).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Factory 4C, read from the edit buffer.
    const P4C: [u8; 21] = [
        0x01, 0x00, 0x01, 0x01, 0x00, 0x05, 0x55, 0x58, 0x40, 0x3d, 0x62, 0x00, 0x00, 0x06, 0x23,
        0x05, 0x01, 0x04, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn parse_4c() {
        let p = Patch::parse(&P4C).unwrap();
        assert!(p.amp_on && !p.mod_on && p.delay_on && p.reverb_on);
        assert_eq!(
            (p.amp_type, p.gain, p.treble, p.middle, p.bass, p.volume),
            (5, 85, 88, 64, 61, 98)
        );
        assert_eq!(p.ir_cab, 4);
        assert_eq!(p.to_bytes(), P4C);
    }

    #[test]
    fn bank_of_36() {
        let mut bank = vec![0xA5, 0x5A];
        for _ in 0..36 {
            bank.extend(P4C);
        }
        assert_eq!(Patch::from_bank(&bank).unwrap().len(), 36);
        assert!(Patch::from_bank(&bank[..100]).is_none());
    }
}
