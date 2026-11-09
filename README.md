# Tank-GL

Native Linux editor for the **M-VAVE Tank-G** pedal, replacing M-EFCS
(Windows only). Rust: protocol in `crates/`, app in egui + fastframe (`app/`,
binary `tank-studio-egui`). The old Tauri + React app was removed; it is in
git history (commit 64065f2).

> Current phase: **reverse engineering + editor.** The pedal answers
> (`TANK-Gv2_097`); see `docs/PROTOCOL.md` section 0.

## App

```bash
cargo build --release -p tank-studio-egui
target/release/tank-studio-egui
```

Edit the current preset on a drawn pedalboard, switch presets and modes,
save and swap presets in flash (with an automatic backup first), and
audition or save IRs and amps from a file, the factory set or
community.m-vave.com. Narrow or portrait windows switch to a vertical layout
with the pedals in two rows.

## CLI

```bash
cargo build --release
target/release/tankg identify          # TANK-Gv2_097
target/release/tankg status            # preset 2A  mode P (preset)
target/release/tankg preset 3C         # or 1–36, next, prev
target/release/tankg backup bank.bin   # preset bank (758 B)
target/release/tankg raw 0x20000000 4  # raw read, debug
```

Works over the USB cable or over **Bluetooth** (`TANK-Gv2_BLE`, paired in
BlueZ). Without the cable, the CLI and the app use Bluetooth on their own;
`tankg --port ble` forces Bluetooth and connects if the pedal is only paired.

ERASE (`0x21`) is blocked in the transport. Flash writes (save, swap, IR/amp
slots) only happen on explicit commands, after a backup of the bank.

## Documents

| Doc | Contents |
|---|---|
| [docs/PLAN.md](docs/PLAN.md) | What the official app does and what ours adds |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Stack, modules, how each feature works, phased roadmap (partly outdated) |
| [docs/PROTOCOL.md](docs/PROTOCOL.md) | How to talk to the Tank-G: USB-MIDI SysEx, BLE, the `00 59` frame, memory map, IR upload, pitfalls |
| [docs/CAPTURES.md](docs/CAPTURES.md) | M-EFCS under Wine, capturing with usbmon, checklist |
| [design/README.md](design/README.md) | Tank Studio design notes, color tokens and icons |

## Summary

- The Tank-G is class-compliant USB-MIDI with SysEx, plus a USB audio
  interface. It works on Linux with no driver. Over Bluetooth LE it speaks the
  same protocol, with the raw frame on the `ae40` service.
- It uses the protocol shared by M-VAVE pedals, which the community already
  documented for the Tank Mini, MK-300, BlackBox and IR Box.
- The pedal has 36 presets, **9 amp slots and 8 IR slots**.
