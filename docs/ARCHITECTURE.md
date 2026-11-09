# Architecture and stack

> **Outdated:** this plan called for a Tauri + React app. The app is now
> written in egui (`app/`) and Tauri was removed. The Rust modules (`crates/`)
> still follow the plan.

## 1. Overview

```
┌──────────────── Tank-GL (Tauri 2) ────────────────┐      ┌──────────────┐
│  Web UI (React + TS)                              │      │ tankg (CLI)  │
│  preset table · chain · library · Ctrl+K          │      │ Hypr binds   │
└──────────────┬────────────────────────────────────┘      └──────┬───────┘
               │ invoke / events (Tauri IPC)                      │ Unix socket
┌──────────────▼──────────────────────────────────────────────────▼───────┐
│  Rust backend (app process = sole owner of the MIDI port)               │
│                                                                         │
│  tankg-engine   state, sync, slot cache, setlists, undo, backup         │
│  tankg-library  SQLite index, scan, hash, tags, IR conversion           │
│  tankg-device   MIDI transport, req/resp, ACK, chunked upload           │
│  tankg-proto    pure, no I/O: frame, pack7, checksum, structs, blobs    │
│  tankg-audio    audition: DI + IR convolution → PipeWire                │
└──────────────┬──────────────────────────────────────────────────────────┘
               │ ALSA sequencer (midir)          │ audio (cpal/PipeWire)
          ┌────▼────────────── Tank-G (USB) ─────▼────┐
          │  USB-MIDI SysEx            USB Audio 2x2  │
          └───────────────────────────────────────────┘
```

Principles:
- **`tankg-proto` is pure.** Only bytes in and out, no I/O. Tested against the
  vectors from the other pedals and, later, against our captures. It is the
  most critical part and the easiest to test.
- **Only one process talks to the pedal at a time.** When the app is open it
  owns the port and the CLI talks to it over a socket. With the app closed,
  the CLI opens the pedal directly (same `tankg-device` crate).
- **The PC is the source of truth.** The pedal keeps a copy: 36 presets, 9
  amps and 8 IRs. Library, history and setlists live on the PC.
- **Flash writes are always explicit.** An automatic backup comes first.
  Dangerous commands (`0x21` ERASE) are blocked by a denylist in the transport
  layer.

## 2. Stack

### Shell and front end

| Piece | Choice | Why |
|---|---|---|
| App | **Tauri 2** | The request. Small binary, Rust backend, WebKitGTK on Linux (already installed: webkit2gtk-4.1 2.52) |
| UI | **React 19 + TypeScript + Vite** | Ecosystem with the two key pieces below |
| Dense table | **TanStack Table + TanStack Virtual** | 36 presets × about 25 parameters, sorting, selection and multi-row editing |
| Ctrl+K palette | **cmdk** | Exactly that component |
| State | **Zustand** + a store derived from backend events | Simple. The backend sends `state-changed` and the UI just reflects it |
| UI undo/redo | Zundo (Zustand middleware) | The real persistent history lives in the backend |
| Look | **Design system in `design/`** (Tank Studio) | `tokens.json` is the source of truth → `tokens.css` (CSS variables). The 22 components in `design/components/` (Knob, SignalChain, PresetGrid, CommandPalette, LibraryTable, MidiMonitor…) are the reference; to be ported to TSX modules in `ui/` |
| Fonts | Barlow, Barlow Condensed, JetBrains Mono | Bundled locally (e.g. `@fontsource`) because the app must work offline, without Google Fonts |
| Styling | CSS with the `tokens.css` variables (Tailwind optional, mapping the tokens into the theme) | One theme: Tank-G |
| Packages | pnpm (11.3 installed) | |

### Rust backend

| Role | Crate | Note |
|---|---|---|
| MIDI | **`midir`** (ALSA seq) | Coexists with PipeWire. Enable SysEx (`Ignore::None`) |
| Finding the pedal | `udev` / sysfs | Match VID:PID, not just the name "Composite"; hotplug |
| Async | `tokio` | Command queue with timeout, ACK and retry |
| Serialization | `serde`, `serde_json` | Presets and setlists in readable JSON |
| Library | **`rusqlite`** (+ FTS5) | Index, tags, favorites, history, slot→hash map |
| Fuzzy search | **`nucleo`** | Helix's matcher: fast with good ranking |
| Hash/dedup | `blake3` | IR/amp identity = content hash |
| Folder watching | `notify` | The IR folder updates itself |
| WAV | `hound` | Reads PCM and float in any format |
| Resampling | **our own implementation** of M-EFCS's Kaiser sinc | Output byte-identical to the editor's. `rubato` only for audition |
| Audition | `cpal` + `realfft`/`fft-convolver` + `symphonia` (DI in mp3/flac) | Local convolution, without writing flash |
| CLI | `clap` | `tankg …` |
| CLI↔app IPC | Unix socket at `$XDG_RUNTIME_DIR/tankg.sock`, line-delimited JSON | |
| Tauri plugins | `dialog`, `fs`, `single-instance`, `global-shortcut` (optional) | |
| Errors and logs | `thiserror`, `anyhow`, `tracing` | Logs with the SysEx hex (TX/RX), useful as a capture |

## 3. Repository layout

```
Tank-GL/
├─ Cargo.toml                 workspace
├─ crates/
│  ├─ tankg-proto/            frame, pack7, checksum, Command/Event, Patch(92B), CabRecord, AmpRecord
│  ├─ tankg-device/           midir, req/resp queue, ACK, upload, denylist, "trace" mode (JSONL)
│  ├─ tankg-library/          SQLite, scan, IR conversion, metadata
│  ├─ tankg-engine/           state, PC↔pedal sync, slot cache, setlists, history, backup
│  ├─ tankg-audio/            audition
│  └─ tankg-cli/              `tankg` binary
├─ src-tauri/                 Tauri app: commands and events over the engine
├─ ui/                        React
├─ design/                    Tank Studio design system (tokens, components, icons)
├─ docs/                      PLAN, ARCHITECTURE, PROTOCOL, CAPTURES
└─ captures/                  local pcaps (out of git) + README with metadata
```

## 4. How each feature works

### 4.1 Connection and state
1. udev detects the Tank-G and the app opens the MIDI port.
2. Handshake: query `11` (model/firmware) and pick the profile by firmware
   (am3/am4).
3. Read the preset bank (family 0) and the current patch (family 1), and
   build the `DeviceState`.
4. From then on, each UI change becomes a **write to the current patch
   (RAM)**, with an instant response. "Save" writes the preset to flash.
5. Changes made on the pedal (footswitch): light polling of the status
   (`0x2000_0000`?) or a notification, to be confirmed.

### 4.2 Slots as a cache (IR and amp)
- The preset on the PC stores `ir_ref = blake3` and `amp_ref = blake3`, not
  slot numbers.
- A `slot_map(kind, slot, hash, last_used, pinned)` table mirrors the pedal.
  It is validated by the name and content read from the slot, because someone
  may have used the official editor in between.
- When loading the preset: if the IR is already on the pedal, just point at
  the slot. If not, write it to the **unpinned LRU slot**, rewrite the
  preset's `ir_slot` and only then select it.
- **Show mode:** once the setlist is chosen, the app works out the IRs and
  amps it needs. If they fit (≤ 8 IRs and ≤ 9 amps), it writes everything
  beforehand and **pins** the slots. Live, nothing is written. If they don't
  fit, the app warns before the show.

### 4.3 Library
- Drag a folder → recursive scan of `.wav` and `.am2data` → hash → SQLite.
  Duplicates are grouped by hash.
- Metadata: tags taken from the file name (speaker, mic, cab size such as
  "412" or "112") plus manual tags and favorites.
- **IR conversion:** any WAV (int/float, mono/stereo, any rate) → mono →
  44.1 kHz (Kaiser sinc) → 2048 int24 samples → `CAB\0` record. The app says
  how much of the IR's tail was cut.

### 4.4 Dense presets
- 36 × parameters table; multi-row editing applies the value to N presets and
  writes them in a batch after one confirmation.
- Unlimited preset library on the PC. "Sync" sends 36 presets to the pedal.
- Side-by-side diff: a structural comparison of the two `Preset`s.
- History: every preset save becomes a version in SQLite. Session undo/redo
  in the front end.
- Full backup: preset bank + 17 AMP/CAB records, in a `.tankgl` file (zip with
  JSON + blobs). Restoring requires confirmation.

### 4.5 CLI (Hyprland)
```
tankg preset 5B            # or tankg preset 14
tankg preset next|prev
tankg bypass toggle
tankg ir load ~/irs/v30.wav [--slot 3] [--yes]
tankg ir list | tankg amp list
tankg backup ~/tankg-$(date +%F).tankgl
tankg monitor              # MIDI monitor with frame decoding
tankg raw read 04 0x10000000 0x5C     # debug, read-only
```
Bind example: `bind = SUPER, F5, exec, tankg preset 1A`.

### 4.6 MIDI monitor
Shows TX/RX with the wire hex, the unpacked frame and a decode (cmd, type,
address, data, checksum ok). Exports JSONL. The same format serves as a test
fixture.

## 5. Roadmap

| Phase | Deliverable | Depends on |
|---|---|---|
| **0. Capture** | Wine + usbmon, the `CAPTURES.md` checklist, decodes in `PROTOCOL.md` | having the pedal |
| **1. Proto + read-only CLI** | `tankg-proto` with tests; `tankg identify`, `dump`, `monitor`, `backup` | phase 0 (handshake/read) |
| **2. Live control** | switch preset, knobs, toggles, bypass, loopback from the CLI | knob capture |
| **3. Basic Tauri app** | effect chain, preset list, save, color, swap, export/import | phase 2 |
| **4. IR/amp** | conversion + upload + factory restore | upload capture |
| **5. Library + slot cache** | scan, search, tags, LRU, show mode | phase 4 |
| **6. Extras** | dense table, Ctrl+K, setlists, diff, history, audition | phase 5 |

Phases 1 and 2 can already be tested with the Tank Mini commands (sec. 5.1 of
`PROTOCOL.md`). If the Tank-G answers the same way, we save weeks.
