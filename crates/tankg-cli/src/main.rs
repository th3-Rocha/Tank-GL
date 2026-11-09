use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use tankg_device::TankG;
use tankg_proto::{Mode, PresetId, preset};

#[derive(Parser)]
#[command(
    name = "tankg",
    about = "Control the M-VAVE Tank-G from Linux",
    version
)]
struct Cli {
    /// Port: usb, ble or part of a MIDI port name. Without it, USB then Bluetooth.
    #[arg(long, global = true)]
    port: Option<String>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Pedal name and firmware
    Identify,
    /// Active preset and mode (P/L)
    Status,
    /// Switch preset: 1A–9D, 1–36, next or prev
    Preset { target: String },
    /// Save the preset bank to a file (read-only on the pedal)
    Backup { file: PathBuf },
    /// Write the current preset (with edits) to flash, into the chosen preset.
    /// Backs up the bank first. E.g.: tankg save 3B --yes
    Save {
        target: String,
        /// Confirm overwriting the preset
        #[arg(long)]
        yes: bool,
    },
    /// Swap two presets in flash. Backs up first. E.g.: tankg swap 1A 1B --yes
    Swap {
        a: String,
        b: String,
        #[arg(long)]
        yes: bool,
    },
    /// Bypass the current preset: on or off
    Bypass { state: String },
    /// Send an IR (WAV) and play it on the pedal. With --slot N --yes, save it to the slot.
    Ir {
        file: PathBuf,
        #[arg(long)]
        slot: Option<u8>,
        /// IR level, 0–255 (M-EFCS uses 50)
        #[arg(long, default_value_t = tankg_proto::block::CAB_DEFAULT_LEVEL)]
        level: u8,
        #[arg(long)]
        yes: bool,
    },
    /// Send an amp (.am4data) and play it on the pedal. With --slot N --yes, save it to the slot.
    Amp {
        file: PathBuf,
        #[arg(long)]
        slot: Option<u8>,
        #[arg(long)]
        yes: bool,
    },
    /// Restore the factory amp or IR (from the installed M-EFCS) to the slot: tankg factory ir 1 --yes
    Factory {
        /// amp or ir
        kind: String,
        slot: u8,
        #[arg(long)]
        yes: bool,
    },
    /// Show live which bytes of the current preset change (turn a knob)
    Watch {
        /// Interval between reads, in ms
        #[arg(long, default_value_t = 150)]
        every: u64,
    },
    /// Change one byte of the current preset (RAM, not saved): tankg set 6 64
    Set {
        /// Preset byte (0–20), see docs/PROTOCOL.md
        offset: usize,
        value: u8,
    },
    /// Switch mode: p (preset) or l (live)
    Mode { mode: String },
    /// Loopback (PC audio comes back through the USB input): on, off, or no argument to show it
    Loopback { state: Option<String> },
    /// Read raw memory (debug): tankg raw 0x20000000 4
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

/// Sends, plays and, if asked, saves to the slot.
fn send_block(
    dev: &mut TankG,
    kind: tankg_proto::block::Kind,
    rec: &[u8],
    slot: Option<u8>,
    yes: bool,
) -> Result<()> {
    if slot.is_some() && !yes {
        bail!(
            "this replaces the {} in that slot on the pedal; repeat with --yes",
            kind.name()
        );
    }
    dev.audition_block(kind, rec, |done, total| {
        eprint!("\rsending {done}/{total} B")
    })?;
    eprintln!();
    println!("playing on the pedal (audition, not saved yet)");
    if let Some(s) = slot {
        dev.save_block(kind, s)?;
        println!("saved to {} slot {s}", kind.name());
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut dev = match cli.port.as_deref() {
        None => TankG::open()?,
        Some("usb") => TankG::open_port(tankg_device::PORT_USB)?,
        Some("ble" | "bt") => TankG::open_bluetooth(true)?,
        Some(p) => TankG::open_port(p)?,
    };

    match cli.cmd {
        Cmd::Identify => println!("{}  ({})", dev.identify()?, dev.port_name()),
        Cmd::Status => {
            let st = dev.status()?;
            let preset = st.preset.map_or("none".into(), |p| p.to_string());
            let mode = match st.mode {
                Mode::Preset => "P (preset)",
                Mode::Live => "L (live)",
            };
            println!("preset {preset}  mode {mode}");
        }
        Cmd::Preset { target } => {
            let p = match target.as_str() {
                "next" | "prev" => {
                    let cur = dev.status()?.preset.map_or(0, PresetId::index);
                    let step = if target == "next" {
                        1
                    } else {
                        preset::COUNT - 1
                    };
                    PresetId::new((cur + step) % preset::COUNT).unwrap()
                }
                t => t.parse().map_err(anyhow::Error::msg)?,
            };
            dev.select_preset(p)?;
            sleep(Duration::from_millis(300));
            let now = dev.status()?.preset;
            if now != Some(p) {
                bail!("asked for {p}, the pedal is on {now:?}");
            }
            println!("preset {p}");
        }
        Cmd::Backup { file } => {
            let bank = dev.read_preset_bank()?;
            if bank[..2] != [0xA5, 0x5A] {
                bail!("bank read does not start with a5 5a: {:02x?}", &bank[..8]);
            }
            std::fs::write(&file, &bank).with_context(|| format!("writing {}", file.display()))?;
            println!("{} bytes → {}", bank.len(), file.display());
        }
        Cmd::Save { target, yes } => {
            let p: PresetId = target.parse().map_err(anyhow::Error::msg)?;
            if !yes {
                bail!("this overwrites {p} in flash; repeat with --yes");
            }
            let rec: [u8; tankg_proto::PATCH_LEN] =
                dev.read_current_patch()?.try_into().expect("21 B");
            let bk = dev.backup_bank(&tankg_device::backup_dir())?;
            println!("backup: {}", bk.display());
            dev.save_preset(p, &rec)?;
            println!("current preset saved to {p} (verified by reading flash back)");
        }
        Cmd::Swap { a, b, yes } => {
            let a: PresetId = a.parse().map_err(anyhow::Error::msg)?;
            let b: PresetId = b.parse().map_err(anyhow::Error::msg)?;
            if !yes {
                bail!("this swaps {a} and {b} in flash; repeat with --yes");
            }
            let bk = dev.backup_bank(&tankg_device::backup_dir())?;
            println!("backup: {}", bk.display());
            dev.swap_presets(a, b)?;
            println!("{a} ↔ {b} swapped (verified by reading flash back)");
        }
        Cmd::Bypass { state } => {
            let st = dev.status()?;
            let on = match state.as_str() {
                "on" => true,
                "off" => false,
                s => bail!("use on or off, not {s:?}"),
            };
            // In bypass the status doesn't give the letter, so "on" stores the
            // preset in a file for "off" to know where to go back.
            let mem = std::env::var_os("XDG_RUNTIME_DIR")
                .map_or_else(std::env::temp_dir, PathBuf::from)
                .join("tankg-bypass");
            let p = match st.preset {
                Some(p) => p,
                None => std::fs::read_to_string(&mem)
                    .ok()
                    .and_then(|s| s.trim().parse::<PresetId>().ok())
                    .filter(|p| p.bank() == st.bank)
                    .context("in bypass without knowing the preset; use tankg preset <id>")?,
            };
            dev.select(p, on)?;
            if on {
                let _ = std::fs::write(&mem, p.to_string());
            }
            println!("bypass {} ({p})", if on { "on" } else { "off" });
        }
        Cmd::Ir {
            file,
            slot,
            level,
            yes,
        } => {
            let bytes =
                std::fs::read(&file).with_context(|| format!("reading {}", file.display()))?;
            let (samples, info) =
                tankg_proto::block::wav_to_cab(&bytes).map_err(anyhow::Error::msg)?;
            println!("{}: {}", file.display(), info.describe());
            let rec = tankg_proto::block::cab_record(&samples, level);
            send_block(&mut dev, tankg_proto::block::Kind::Cab, &rec, slot, yes)?;
        }
        Cmd::Amp { file, slot, yes } => {
            let rec =
                std::fs::read(&file).with_context(|| format!("reading {}", file.display()))?;
            println!(
                "{}: amp {:?}",
                file.display(),
                tankg_proto::block::record_name(&rec)
            );
            send_block(&mut dev, tankg_proto::block::Kind::Amp, &rec, slot, yes)?;
        }
        Cmd::Factory { kind, slot, yes } => {
            use tankg_proto::block::Kind;
            let kind = match kind.as_str() {
                "amp" => Kind::Amp,
                "ir" | "cab" => Kind::Cab,
                k => bail!("use amp or ir, not {k:?}"),
            };
            let path = tankg_device::find_factory_bank()
                .context("M-EFCS not found (tankG_am4_ampCab.bin)")?;
            let bank = std::fs::read(&path)?;
            let rec = tankg_proto::block::factory_record(&bank, kind, slot)
                .context("invalid slot or unexpected factory bank")?;
            println!(
                "factory {} for slot {slot}: {:?}",
                kind.name(),
                tankg_proto::block::record_name(&rec)
            );
            send_block(&mut dev, kind, &rec, Some(slot), yes)?;
        }
        Cmd::Set { offset, value } => {
            let before = dev.read_current_patch()?;
            dev.write_current_patch(offset, &[value])?;
            sleep(Duration::from_millis(100));
            let after = dev.read_current_patch()?;
            println!(
                "byte {offset}: {} → {} (asked for {value})",
                before[offset], after[offset]
            );
            if after[offset] != value {
                bail!("the pedal ACKed but the value did not change");
            }
        }
        Cmd::Mode { mode } => {
            let m = match mode.to_ascii_lowercase().as_str() {
                "p" | "preset" => Mode::Preset,
                "l" | "live" => Mode::Live,
                s => bail!("use p or l, not {s:?}"),
            };
            dev.set_mode(m)?;
            sleep(Duration::from_millis(200));
            let now = dev.status()?.mode;
            if now != m {
                bail!("asked for {m:?}, the pedal is in {now:?}");
            }
            println!(
                "mode {}",
                if m == Mode::Preset {
                    "P (preset)"
                } else {
                    "L (live)"
                }
            );
        }
        Cmd::Loopback { state } => {
            match state.as_deref() {
                None => {}
                Some("on") => dev.set_loopback(true)?,
                Some("off") => dev.set_loopback(false)?,
                Some(s) => bail!("use on or off, not {s:?}"),
            }
            println!("loopback {}", if dev.loopback()? { "on" } else { "off" });
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

/// Prints the whole current preset on the first read and afterwards only
/// what changed: `byte 7: 58 → 5c (88 → 92)`.
fn watch(dev: &mut TankG, every: Duration) -> Result<()> {
    let mut last: Option<(Option<PresetId>, Vec<u8>)> = None;
    loop {
        // A missed read (pedal busy, mode change) doesn't stop the watch.
        let (st, patch) = match dev
            .status()
            .and_then(|st| Ok((st, dev.read_current_patch()?)))
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("(read failed: {e}; trying again)");
                sleep(every);
                continue;
            }
        };
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis()
            % 100_000;
        match &last {
            Some((p, prev)) if *p == st.preset => {
                for (i, (a, b)) in prev.iter().zip(&patch).enumerate() {
                    if a != b {
                        println!(
                            "{:>3}.{:03}s  byte {i:>2}: {a:02x} → {b:02x}  ({a} → {b})",
                            t / 1000,
                            t % 1000
                        );
                    }
                }
            }
            _ => {
                let p = st.preset.map_or("none".into(), |p| p.to_string());
                let hex: Vec<_> = patch.iter().map(|b| format!("{b:02x}")).collect();
                println!("preset {p}: {}", hex.join(" "));
            }
        }
        last = Some((st.preset, patch));
        sleep(every);
    }
}
