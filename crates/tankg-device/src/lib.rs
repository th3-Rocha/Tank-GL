//! Conversa com o Tank-G pela porta USB-MIDI (ALSA sequencer via midir).

use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use tankg_proto::{self as proto, Frame, PresetId, Status, cmd};
use thiserror::Error;

/// O Tank-G aparece como Jieli genérico (USB `4c4a:c755`).
const PORT_HINT: &str = "USB Composite Device";
const CLIENT: &str = "tankg";
const TIMEOUT: Duration = Duration::from_millis(1500);
/// Leituras maiores voltam vazias; 100 B por pedido funciona.
const READ_CHUNK: usize = 100;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Tank-G não encontrado (porta MIDI com \"{PORT_HINT}\")")]
    NotFound,
    #[error("porta MIDI ocupada ou indisponível: {0}")]
    Port(String),
    #[error("sem resposta do pedal em {0:?}")]
    Timeout(Duration),
    #[error("comando 0x{0:02x} bloqueado")]
    Denied(u8),
    #[error(transparent)]
    Proto(#[from] proto::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct TankG {
    out: MidiOutputConnection,
    _in: MidiInputConnection<()>,
    rx: Receiver<Vec<u8>>,
    port_name: String,
}

impl TankG {
    pub fn open() -> Result<Self> {
        let port_err = |e: &dyn std::fmt::Display| Error::Port(e.to_string());

        let mut input = MidiInput::new(CLIENT).map_err(|e| port_err(&e))?;
        input.ignore(Ignore::None);
        let in_port = input
            .ports()
            .into_iter()
            .find(|p| input.port_name(p).is_ok_and(|n| n.contains(PORT_HINT)))
            .ok_or(Error::NotFound)?;
        let port_name = input.port_name(&in_port).unwrap_or_default();

        let output = MidiOutput::new(CLIENT).map_err(|e| port_err(&e))?;
        let out_port = output
            .ports()
            .into_iter()
            .find(|p| output.port_name(p).is_ok_and(|n| n.contains(PORT_HINT)))
            .ok_or(Error::NotFound)?;

        let (tx, rx) = mpsc::channel();
        let mut pending = Vec::new();
        let conn_in = input
            .connect(
                &in_port,
                "tankg-in",
                move |_, bytes, _| {
                    // O ALSA pode entregar um SysEx longo em pedaços.
                    if bytes.first() == Some(&0xF0) {
                        pending.clear();
                    }
                    pending.extend_from_slice(bytes);
                    if pending.last() == Some(&0xF7) && pending.first() == Some(&0xF0) {
                        let _ = tx.send(std::mem::take(&mut pending));
                    }
                },
                (),
            )
            .map_err(|e| port_err(&e))?;
        let out = output.connect(&out_port, "tankg-out").map_err(|e| port_err(&e))?;

        Ok(Self { out, _in: conn_in, rx, port_name })
    }

    pub fn port_name(&self) -> &str {
        &self.port_name
    }

    fn send_raw(&mut self, msg: &[u8]) -> Result<()> {
        self.out.send(msg).map_err(|e| Error::Port(e.to_string()))
    }

    /// Manda um frame e espera a primeira resposta SysEx válida.
    pub fn request(&mut self, frame: &Frame) -> Result<Frame> {
        self.request_where(frame, |_| true)
    }

    /// Como [`request`](Self::request), mas ignora respostas que não passam
    /// em `accept`. Outro programa (o app aberto) pode estar lendo ao mesmo
    /// tempo, e o ALSA entrega as respostas dele para nós também.
    fn request_where(&mut self, frame: &Frame, accept: impl Fn(&Frame) -> bool) -> Result<Frame> {
        // Denylist: só leitura por enquanto. Escrita entra quando estiver capturada.
        if frame.cmd == cmd::ERASE || frame.cmd == cmd::WRITE {
            return Err(Error::Denied(frame.cmd));
        }
        while self.rx.try_recv().is_ok() {}
        self.send_raw(&frame.to_sysex())?;
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let msg = self.rx.recv_timeout(left).map_err(|_| Error::Timeout(TIMEOUT))?;
            if let Ok(resp) = Frame::from_sysex(&msg)
                && accept(&resp)
            {
                return Ok(resp);
            }
        }
    }

    pub fn identify(&mut self) -> Result<String> {
        let resp = self.request(&proto::identity_query())?;
        Ok(proto::parse_identity(&resp)?)
    }

    pub fn read(&mut self, addr: u32, len: usize) -> Result<Vec<u8>> {
        let mut data = Vec::with_capacity(len);
        while data.len() < len {
            let off = data.len();
            let n = READ_CHUNK.min(len - off);
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

    pub fn select_preset(&mut self, p: PresetId) -> Result<()> {
        self.send_raw(&proto::program_change(p))
    }

    /// Preset atual (edit buffer, RAM), um registro de 21 B.
    pub fn read_current_patch(&mut self) -> Result<Vec<u8>> {
        self.read(proto::addr::CURRENT_PATCH, proto::PATCH_LEN)
    }

    pub fn read_preset_bank(&mut self) -> Result<Vec<u8>> {
        self.read(proto::addr::PRESET_BANK, proto::PRESET_BANK_LEN)
    }
}
