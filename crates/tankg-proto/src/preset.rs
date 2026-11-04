//! Identificação de preset: 9 bancos × 4 (A–D) = 36.

use std::fmt;
use std::str::FromStr;

pub const BANKS: u8 = 9;
pub const SLOTS: u8 = 4;
pub const COUNT: u8 = BANKS * SLOTS;

/// Preset `0..36`. Mostrado como `1A`…`9D`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PresetId(u8);

impl PresetId {
    pub fn new(index: u8) -> Option<Self> {
        (index < COUNT).then_some(Self(index))
    }

    /// `bank` e `slot` começam em 0, como no registrador de estado.
    pub fn from_bank_slot(bank: u8, slot: u8) -> Option<Self> {
        (bank < BANKS && slot < SLOTS).then(|| Self(bank * SLOTS + slot))
    }

    pub fn index(self) -> u8 {
        self.0
    }

    pub fn bank(self) -> u8 {
        self.0 / SLOTS
    }

    pub fn slot(self) -> u8 {
        self.0 % SLOTS
    }
}

impl fmt::Display for PresetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.bank() + 1, (b'A' + self.slot()) as char)
    }
}

impl FromStr for PresetId {
    type Err = String;

    /// Aceita `2B` / `2b` ou o número `1..=36`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let err = || format!("preset inválido: {s:?} (use 1A–9D ou 1–36)");
        if let Ok(n) = s.parse::<u8>() {
            return n.checked_sub(1).and_then(Self::new).ok_or_else(err);
        }
        let (bank, slot) = s.split_at(s.len().checked_sub(1).ok_or_else(err)?);
        let bank: u8 = bank.parse().map_err(|_| err())?;
        let slot = match slot.to_ascii_uppercase().as_str() {
            "A" => 0,
            "B" => 1,
            "C" => 2,
            "D" => 3,
            _ => return Err(err()),
        };
        bank.checked_sub(1).and_then(|b| Self::from_bank_slot(b, slot)).ok_or_else(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_display() {
        for (s, idx) in [("1A", 0), ("2b", 5), ("4B", 13), ("9D", 35), ("1", 0), ("36", 35)] {
            let p: PresetId = s.parse().unwrap();
            assert_eq!(p.index(), idx, "{s}");
        }
        assert_eq!(PresetId::new(13).unwrap().to_string(), "4B");
        for bad in ["0", "37", "10A", "1E", "", "A"] {
            assert!(bad.parse::<PresetId>().is_err(), "{bad}");
        }
    }
}
