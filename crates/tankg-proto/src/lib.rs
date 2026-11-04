//! Protocolo do M-VAVE Tank-G. Só bytes, sem I/O.
//!
//! Frame bruto: `00 59 | cmd | len u24 LE | corpo | cs`, com
//! `cs = !soma(corpo)`. No USB ele vai empacotado em 7 bits (LSB primeiro)
//! dentro de `F0 … F7`. Ver `docs/PROTOCOLO.md`.

pub mod pack;
pub mod patch;
pub mod preset;

pub use pack::{pack7, unpack7};
pub use patch::Patch;
pub use preset::PresetId;

use thiserror::Error;

pub const MAGIC: [u8; 2] = [0x00, 0x59];
const HEADER_LEN: usize = 6;

/// Comandos do frame bruto.
pub mod cmd {
    pub const ACK: u8 = 0x00;
    pub const IDENTITY: u8 = 0x11;
    /// ⛔ Nunca enviar. A camada de transporte recusa.
    pub const ERASE: u8 = 0x21;
    pub const WRITE: u8 = 0x22;
    pub const READ: u8 = 0x23;
}

/// Tipo de memória usado em READ/WRITE.
pub const MEM_DEV: u8 = 0x04;

/// Endereços confirmados no Tank-G (fw `TANK-Gv2_097`).
pub mod addr {
    /// Banco de presets na flash (começa com `a5 5a`).
    pub const PRESET_BANK: u32 = 0x0000_0000;
    /// Preset atual (edit buffer). Ler com exatamente [`PATCH_LEN`](super::PATCH_LEN)
    /// bytes; outro tamanho devolve buffer velho.
    pub const CURRENT_PATCH: u32 = 0x1000_0000;
    /// Estado: `[banco no display, banco ativo, letra, modo]`.
    pub const STATUS: u32 = 0x2000_0000;
}

/// Um preset: 21 bytes, no banco e no edit buffer.
pub const PATCH_LEN: usize = 21;

/// Banco de presets: `a5 5a` + 36 registros de [`PATCH_LEN`]
/// (igual a `tankG_am4_preset.bin`).
pub const PRESET_BANK_LEN: usize = 2 + preset::COUNT as usize * PATCH_LEN;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("SysEx sem F0/F7")]
    NotSysex,
    #[error("frame curto ({0} bytes)")]
    Short(usize),
    #[error("magic inválido: {0:02x?}")]
    BadMagic([u8; 2]),
    #[error("checksum errado: veio {got:02x}, esperado {want:02x}")]
    Checksum { got: u8, want: u8 },
    #[error("resposta não bate com o pedido")]
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
        Self { cmd, body: body.into() }
    }

    /// Frame bruto (8 bits), sem empacotar.
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

    /// Mensagem SysEx pronta para o fio.
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

/// Cabeçalho de READ/WRITE: `tipo addr(u32 LE) len(u24 LE)`.
fn mem_header(addr: u32, len: usize) -> Vec<u8> {
    let mut h = vec![MEM_DEV];
    h.extend_from_slice(&addr.to_le_bytes());
    h.extend_from_slice(&(len as u32).to_le_bytes()[..3]);
    h
}

pub fn read_request(addr: u32, len: usize) -> Frame {
    Frame::new(cmd::READ, mem_header(addr, len))
}

/// Tira o eco do pedido da resposta de um READ e devolve os dados.
///
/// O pedal responde a leituras fora do mapa com o buffer velho, então o
/// tamanho é conferido aqui; o conteúdo, quem chama confere.
pub fn parse_read_response(resp: &Frame, addr: u32, len: usize) -> Result<Vec<u8>, Error> {
    let header = mem_header(addr, len);
    if resp.cmd != cmd::READ || !resp.body.starts_with(&header) || resp.body.len() != header.len() + len {
        return Err(Error::Mismatch);
    }
    Ok(resp.body[header.len()..].to_vec())
}

/// Nome e firmware da resposta ao `0x11` (ex.: `TANK-Gv2_097`).
pub fn parse_identity(resp: &Frame) -> Result<String, Error> {
    if resp.cmd != cmd::IDENTITY {
        return Err(Error::Mismatch);
    }
    let end = resp.body.iter().position(|&b| b == 0).unwrap_or(resp.body.len());
    Ok(String::from_utf8_lossy(&resp.body[..end]).into_owned())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Preset,
    Live,
}

/// Conteúdo de [`addr::STATUS`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Status {
    /// Banco mostrado no display (pode piscar antes de confirmar). Hipótese.
    pub display_bank: u8,
    /// `None` quando nenhum preset foi selecionado desde o boot (`ff`).
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
            preset: PresetId::from_bank_slot(bank, slot),
            mode: if mode == 0 { Mode::Live } else { Mode::Preset },
        })
    }
}

/// Program Change que seleciona o preset (canal 1).
pub fn program_change(p: PresetId) -> [u8; 2] {
    [0xC0, p.index()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(s: &str) -> Vec<u8> {
        s.split_whitespace().map(|b| u8::from_str_radix(b, 16).unwrap()).collect()
    }

    #[test]
    fn identity_query_wire() {
        assert_eq!(identity_query().to_sysex(), hex("F0 00 32 45 00 00 00 40 7F 00 F7"));
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
        let data = parse_read_response(&Frame::new(cmd::READ, body), addr::STATUS, Status::LEN).unwrap();
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
    fn bad_checksum_rejected() {
        let mut raw = Frame::new(cmd::ACK, [0x00]).to_raw();
        *raw.last_mut().unwrap() ^= 1;
        assert!(matches!(Frame::from_raw(&raw), Err(Error::Checksum { .. })));
    }
}
