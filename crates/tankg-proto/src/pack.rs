//! M-VAVE 7-bit packing: a continuous bit stream, LSB first.
//! Not the "one MSB byte every 7" scheme.

/// `n` bytes become `n + n/7 + 1`; the final flush byte is always sent.
pub fn pack7(raw: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(raw.len() + raw.len() / 7 + 1);
    let (mut acc, mut nbits) = (0u32, 0);
    for &b in raw {
        acc |= (b as u32) << nbits;
        nbits += 8;
        while nbits >= 7 {
            out.push((acc & 0x7F) as u8);
            acc >>= 7;
            nbits -= 7;
        }
    }
    out.push((acc & 0x7F) as u8);
    out
}

/// Inverse of [`pack7`]. Junk may be left at the end; cut by the frame's
/// `len`, never by stripping zeros (the checksum can be `00`).
pub fn unpack7(wire: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(wire.len() * 7 / 8);
    let (mut acc, mut nbits) = (0u32, 0);
    for &b in wire {
        acc |= ((b & 0x7F) as u32) << nbits;
        nbits += 7;
        if nbits >= 8 {
            out.push(acc as u8);
            acc >>= 8;
            nbits -= 8;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_all_lengths() {
        for n in 0..64 {
            let raw: Vec<u8> = (0..n).map(|i| (i * 37 + 11) as u8).collect();
            let wire = pack7(&raw);
            assert_eq!(wire.len(), n + n / 7 + 1);
            assert!(wire.iter().all(|&b| b < 0x80));
            assert_eq!(&unpack7(&wire)[..n], &raw[..]);
        }
    }
}
