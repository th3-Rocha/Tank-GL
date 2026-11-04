use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use tankg_device::TankG;
use tankg_proto::{Mode, PresetId, preset};

#[derive(Parser)]
#[command(name = "tankg", about = "Controle do M-VAVE Tank-G pelo Linux", version)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Nome e firmware do pedal
    Identify,
    /// Preset ativo e modo (P/L)
    Status,
    /// Troca de preset: 1A–9D, 1–36, next ou prev
    Preset { target: String },
    /// Salva o banco de presets num arquivo (só leitura no pedal)
    Backup { file: PathBuf },
    /// Mostra ao vivo os bytes do preset atual que mudam (gire um knob)
    Watch {
        /// Intervalo entre leituras, em ms
        #[arg(long, default_value_t = 150)]
        every: u64,
    },
    /// Lê memória crua (debug): tankg raw 0x20000000 4
    Raw {
        #[arg(value_parser = parse_u32)]
        addr: u32,
        len: usize,
    },
}

fn parse_u32(s: &str) -> Result<u32, String> {
    let r = match s.strip_prefix("0x") {
        Some(h) => u32::from_str_radix(h, 16),
        None => s.parse(),
    };
    r.map_err(|e| e.to_string())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut dev = TankG::open()?;

    match cli.cmd {
        Cmd::Identify => println!("{}  ({})", dev.identify()?, dev.port_name()),
        Cmd::Status => {
            let st = dev.status()?;
            let preset = st.preset.map_or("nenhum".into(), |p| p.to_string());
            let mode = match st.mode {
                Mode::Preset => "P (preset)",
                Mode::Live => "L (live)",
            };
            println!("preset {preset}  modo {mode}");
        }
        Cmd::Preset { target } => {
            let p = match target.as_str() {
                "next" | "prev" => {
                    let cur = dev.status()?.preset.map_or(0, PresetId::index);
                    let step = if target == "next" { 1 } else { preset::COUNT - 1 };
                    PresetId::new((cur + step) % preset::COUNT).unwrap()
                }
                t => t.parse().map_err(anyhow::Error::msg)?,
            };
            dev.select_preset(p)?;
            sleep(Duration::from_millis(300));
            let now = dev.status()?.preset;
            if now != Some(p) {
                bail!("pedi {p}, o pedal está em {now:?}");
            }
            println!("preset {p}");
        }
        Cmd::Backup { file } => {
            let bank = dev.read_preset_bank()?;
            if bank[..2] != [0xA5, 0x5A] {
                bail!("banco lido não começa com a5 5a: {:02x?}", &bank[..8]);
            }
            std::fs::write(&file, &bank).with_context(|| format!("gravando {}", file.display()))?;
            println!("{} bytes → {}", bank.len(), file.display());
        }
        Cmd::Watch { every } => watch(&mut dev, Duration::from_millis(every))?,
        Cmd::Raw { addr, len } => {
            let data = dev.read(addr, len)?;
            for (i, row) in data.chunks(16).enumerate() {
                let hex: Vec<_> = row.iter().map(|b| format!("{b:02x}")).collect();
                println!("{:08x}  {}", addr as usize + i * 16, hex.join(" "));
            }
        }
    }
    Ok(())
}

/// Imprime o preset atual inteiro na primeira leitura e depois só o que
/// mudou: `byte 7: 58 → 5c (88 → 92)`.
fn watch(dev: &mut TankG, every: Duration) -> Result<()> {
    let mut last: Option<(Option<PresetId>, Vec<u8>)> = None;
    loop {
        // Uma leitura perdida (pedal ocupado, troca de modo) não para o watch.
        let (st, patch) = match dev.status().and_then(|st| Ok((st, dev.read_current_patch()?))) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("(leitura falhou: {e}; tentando de novo)");
                sleep(every);
                continue;
            }
        };
        let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_millis() % 100_000;
        match &last {
            Some((p, prev)) if *p == st.preset => {
                for (i, (a, b)) in prev.iter().zip(&patch).enumerate() {
                    if a != b {
                        println!("{:>3}.{:03}s  byte {i:>2}: {a:02x} → {b:02x}  ({a} → {b})", t / 1000, t % 1000);
                    }
                }
            }
            _ => {
                let p = st.preset.map_or("nenhum".into(), |p| p.to_string());
                let hex: Vec<_> = patch.iter().map(|b| format!("{b:02x}")).collect();
                println!("preset {p}: {}", hex.join(" "));
            }
        }
        last = Some((st.preset, patch));
        sleep(every);
    }
}
