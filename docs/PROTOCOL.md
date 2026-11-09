# Tank-G protocol: what we know and what still needs confirming

> State on 2026-10-17. The Tank-G had **not been plugged in or captured** yet.
> Everything here comes from three sources: (1) the official editor's binary
> `M-EFCS.zip`, (2) published reverse engineering of other M-VAVE pedals that
> use the same editor and (3) inference from those two.
>
> Markers used in the text:
> - ✅ **verified**: it is in the M-EFCS binary or was measured on another pedal of the family
> - 🟡 **strong hypothesis**: the pattern is the same on 3 or more M-VAVE pedals
> - ❓ **to capture**: only a capture on the Tank-G settles it
> - 🎸 **measured on the Tank-G**: read on the real pedal (section 0)

---

## 0. Measured on the Tank-G (2026-11-04)

First plug, read-only plus preset switching. Implemented in
`crates/tankg-proto` and in the `tankg` CLI.

| Item | Result | |
|---|---|---|
| USB | `4c4a:c755` "Jieli Technology USB Composite Device", serial `4150353336323716`. **Same VID:PID as the BlackBox** | 🎸 |
| MIDI port | ALSA `hw:N,0,0` "USB Composite Device MIDI 1" (the kernel warns "falling back to MIDI 1.0", which is normal) | 🎸 |
| Identity (`11`) | `TANK-Gv2_097` + zeros (27 B). Frame, checksum and 7-bit packing are the same as the family's | 🎸 |
| `04 / 0x0000_0000` | **Preset bank, 758 B**, starts with `a5 5a`. Identical to M-EFCS's `tankG_am4_preset.bin`, except bytes 0–20 (header, probably global config). The firmware is from the **am4** line | 🎸 |
| `04 / 0x2000_0000` | **Status, 4 B:** `[bank?, active bank, letter A–D (0–3, ff = none since boot), mode (01 = P, 00 = L)]`. Bytes 0 and 1 were always equal; byte 0 is probably the bank blinking on the display before it is confirmed. Banks count from 0 | 🎸 |
| `04 / 0xD000_0000` | `01`. Probably Loopback, not confirmed | 🟡 |
| `04 / 0xE000_0000` | `1a`, doesn't change when switching presets. **Not the active preset** on the Tank-G | 🎸 |
| `04 / 0x1000_0000` | Well-formed reply with a **stale buffer** inside. Not the current patch on the Tank-G | 🎸 |
| Program Change `C0 n` | Selects preset `n` (0–35 = 1A–9D), confirmed through the status | 🎸 |
| Read size | 100 B requests work. **Reading past the end of the bank returns a stale buffer** with a valid checksum (pitfall 1 in section 7) | 🎸 |

Preset map: 9 banks × 4 (A–D) = 36, `index = bank·4 + letter`.

**Bank format** 🎸: `a5 5a` + **36 records of 21 B** (2 + 36·21 = 758), in
1A…9D order. On this pedal 1A is all zeros; the other 35 match the factory
ones.

**Current preset (edit buffer)** 🎸: `04 / 0x1000_0000`, **read exactly 21 B**
(it is the same record as in the bank). In LIVE mode it changes immediately
when a knob is turned or a footswitch pressed, without saving. Log:
`captures/knobs-4C-2026-11-04.log`.

**21-byte record** 🎸 (knobs follow the panel's order):

| Byte | Field | Range |
|---|---|---|
| 0 | AMP on/off (footswitch D in LIVE) | 0/1 |
| 1 | MOD on/off (C) | 0/1 |
| 2 | DELAY on/off (B) | 0/1 |
| 3 | REVERB on/off (A) | 0/1 |
| 4 | NOISE GATE | 0–127 |
| 5 | AMP TYPE | 0–8 |
| 6 | GAIN | 0–127 |
| 7 | TREBLE | 0–127 |
| 8 | MIDDLE | 0–127 |
| 9 | BASS | 0–127 |
| 10 | VOLUME | 0–127 |
| 11 | MOD FX (position; the zone picks the type) | 0–127 |
| 12 | MOD SPEED | 0–127 |
| 13 | DLY MIX (position; the zone picks the type) | 0–127 |
| 14 | DLY TIME | 0–127 |
| 15 | RVB DECAY (position; the zone picks the type) | 0–127 |
| 16 | RVB MIX | 0–127 |
| 17 | IR CAB (0 = OFF, 1–8 = slot) | 0–8 |
| 18 | ? factory `ff` (24 presets) or `00` (12). LED color? | ❓ |
| 19–20 | ? always `00` | ❓ |

**MASTER is not in the preset**: turning it changes no byte. It is global.

**Writes** 🎸 (`22`: `04 addr u32 len u24 data`, the pedal answers with the ACK `00 59 00 01 00 00 00 FF`):

| Address | Effect | |
|---|---|---|
| `0x1000_0000 + byte` | Changes the current preset's field immediately, in P and L modes. It is RAM: **sending the same preset again (Program Change) discards it** and goes back to the saved one | 🎸 |
| `0x2000_0003` | Mode: `01` = PRESET, `00` = LIVE. The pedal really changes behavior (footswitches start toggling effects) | 🎸 |
| `0xD000_0000` | Loopback, `00` turns it off. Measured on 2026-11-05: with a 1 kHz tone playing on the PC, the pedal's USB capture has 1 kHz at −24 dBFS when on and nothing (−110 dBFS) when off. Doesn't affect the pedal's headphone/line output | ✅ |

Captured from M-EFCS on 2026-11-09 with usbmon (`tools/usbmon_tankg.py`, log
in `captures/mefcs-save-2026-11-09.log`). Type `04` = presets and status,
type `05` = amp and IR blocks.

**Presets (type `04`)** 🎸

| Action in M-EFCS | Frames |
|---|---|
| Turning a knob | WRITE `0x1000_0004`, 14 B (bytes 4–17 of the current preset at once) |
| **Save preset to N** | WRITE `0x0000_0002 + N·21` with the 21 B → ACK; WRITE `0xF000_0000` with no data (flash **commit**); WRITE `0xE000_0000 + N` = `01` (selects N). Checked by reading the bank back: only record N changed |
| **Swap A ↔ B** | Two record WRITEs (each into the other's place), **one** commit, selects the target. Checked in the bank |
| Select preset | WRITE `0xE000_0000 + N` = `01` |
| **Bypass on** | WRITE `0xE100_0000 + N` = `01` (N = current preset; the pedal plays N without effects) |
| **Bypass off** | WRITE `0xE000_0000 + N` = `01` (selects the preset again) |
| Loopback | WRITE `0xD000_0000` = `01`/`00` |
| On connect | IDENT; the whole bank in 100 B reads; READ type `05` `0xD000_0000` 32 B (`0a "097" …`, firmware info) |

**IR (CAB) and amp (type `05`)** 🎸

IR record: **6164 B** = `CAB\0` + name (14 B, M-EFCS sends it empty) + `01` +
**level** (byte 19, 0–255, M-EFCS starts at `0x32`) + **2048 int24 LE
samples** (44.1 kHz). The WAV samples go in unconverted: confirmed with
markers at known positions (`captures/ir-teste-impulso.wav`).

Amp record: **8192 B** = `AM4\0` + name (14 B) + byte 18 (`64`) + **Level**
(byte 19) + **Gain** (bytes 20–21, two values) + neural network. It is the
same format as the 8 KiB of each slot in `tankG_am4_ampCab.bin`, and the
`.am4data` file M-EFCS imports is that raw record.

| Step | Frames |
|---|---|
| Send the record | WRITE type `05` addr = offset, chunks of **1015 B** (IR: 7 chunks, amp: 9) |
| **Audition** (play without saving) | WRITE `0xA000_0000` = `02` (IR) or `01` (amp) |
| Level of the auditioned IR | WRITE `0x2000_0013` = level (1 B) |
| Level of the auditioned amp | WRITE `0x1000_0013` = level (1 B) |
| Gain of the auditioned amp | WRITE `0x1000_0014`, 2 B |
| **Save to slot** | IR: WRITE `0xF000_0100` = slot−1. Amp: WRITE `0xF000_0000` = slot−1. Saves the last auditioned record; for an amp M-EFCS resends the record first. ACK right away |

So, in type `05`: family `0x1…` = auditioned amp, `0x2…` = auditioned IR,
`0xA…` = audition, `0xF000_0000 + 0x100·type` = save slot.

Community amps come in varied sizes (e.g. 7868 B) and the pedal takes them:
tested in audition on 2026-11-09. 14 KB files are for other pedals.

**Community** (community.m-vave.com, M-EFCS's Community tab): public listing
and files, no account. `POST /bbs/api/getlist` with
`{search, select: [], sort: 0, currentPage, pageSize}` returns posts with
`fileType` (`am4data`, `wav`, `am3data`, `kz300`…) and `filePath` (JSON with
the server's `fileName`). The file comes from
`GET /bbs/api/audition/model-file?name=<fileName>`.

**Bluetooth** (2026-11-09) 🎸: `TANK-Gv2_BLE` has two control services.
`TANK-Gv2` (without `_BLE`) is Bluetooth audio only.

| Path | Result |
|---|---|
| Standard BLE-MIDI (`03b80e5a…`), through the ALSA port BlueZ creates | Program Change works; **SysEx is ignored** (neither executed nor answered) |
| Service `ae40`: write to `ae41` (write-without-response only), notifications on `ae42` | **Works like USB, with the raw frame** (no F0/F7, no 7-bit packing). Replies and ACKs come back on `ae42` |

`ae40` details:

- Notifications have to be enabled over D-Bus (`StartNotify` on `ae42` and
  listening to `PropertiesChanged`). Through `bluetoothctl` the replies didn't
  show up, which is what misled the first test.
- MTU 517. A 400 B READ reply arrives in a single notification (on USB the
  limit is 100 B per request).
- A frame bigger than one packet (MTU − 3 = 514 B) can go in several writes:
  the pedal joins them and answers a single ACK (tested with the IR's 1015 B
  chunk in 3 packets). A packet above 514 B fails in BlueZ, and the pedal
  keeps waiting for the rest of the cut frame: the following commands are
  swallowed.
- Write with response (`request`) is refused: "Write not permitted".
- Implemented in `crates/tankg-device/src/ble.rs`. `TankG::open()` tries USB
  and then Bluetooth (if already connected); `tankg --port ble` connects if
  the pedal is only paired.

M-EFCS has no LED color for the Tank-G. Still open: confirm that saved IRs and
amps survive powering the pedal off, and what the pedal does with an IR's
empty name.

To do in Tank Studio: save preset (backup before, read back after), swap,
Bypass, send IR (WAV → record) and amp (`.am4data`). Implemented in
`crates/tankg-proto/src/patch.rs`.

The "names" M-EFCS shows (`GENUL`…) are those bytes viewed as ASCII, not
names.

**Battery** 🎸: not found. M-EFCS doesn't read the Tank-G's battery
(`RcLowBattery` belongs to another product). Reading `0x2000_0000` with more
than 4 B, or families `0x3…`–`0xC…`, returns a stale buffer. Over USB it is
always charging; if there is a reading, it must be over BLE.

Still open: confirm the record's fields, where the current patch's
knobs/toggles live, LED color, IR Lv, writing and upload.

## 1. Summary

1. The Tank-G talks **class-compliant USB-MIDI with SysEx**. On Linux it should
   show up as an ALSA port with no driver at all (`snd-usb-audio`). It is also
   a USB audio interface, which is where the Loopback feature comes from.
2. All recent M-VAVE pedals use the same raw frame
   **`00 59 | cmd | len u24 | body | checksum`**. Over USB that frame is packed
   into 7 bits inside `F0 … F7`. Since `00 59` packed becomes `00 32`, every
   message starts with `F0 00 32`.
3. The official editor is a **Flutter** app (`m_efcs`) with Tank-G-specific
   code (`data/products/tank_g/structs.dart`, `effect_params.dart`,
   `transport/midi/sysex_codec.dart`). It bundles the **Tank-G factory
   banks**: 9 amp slots and 8 CAB slots.
4. The **Tank Mini** (the smaller sibling) has its whole protocol documented:
   preset, knobs, toggles, 92-byte dump and flash save. It uses exactly the
   same memory addressing seen on the BlackBox. The working hypothesis is that
   the Tank-G follows the same model.
5. **IR upload** (`CAB\0` record, 2048 int24 samples at 44.1 kHz) has already
   been reproduced **byte for byte** on the MK-300 and the BlackBox. The
   Tank-G has CAB slots of the same kind.
6. **M-EFCS runs on Linux under Wine** with a 1-byte patch. That allows
   capturing directly on Linux with `usbmon`, no VM (see `CAPTURES.md`).

---

## 2. Evidence from the official binary (`M-EFCS.zip`, 2026-10-10 build)

| Item | Finding | |
|---|---|---|
| Framework | Flutter/Dart AOT (`data/app.so`), plugins `flutter_midi_command_windows`, `universal_ble` | ✅ |
| Transport | `transport/midi/sysex_codec.dart`, classes `SysExCodec` and `SysExStreamDecoder`; also BLE-MIDI (`BLEMidiDevice`) | ✅ |
| Tank-G code | `products/tank_g/{config,data,effect_params,structs}.dart`, `ui/products/tank_g_view.dart`, `buildTankGEffectParams` | ✅ |
| Function strings | `setLoopbackEnabled`, `swapPreset`, `SwapPresetToDialog`, `presetSaved`, `am2data`, `ImportCabWidget`, `utils/wave_utils.dart` | ✅ |
| Factory banks | `flutter_assets/lib/resources/bin/tankG/` (details below) | ✅ |

### 2.1 Bundled factory banks

`tankG_am3_*` and `tankG_am4_*` are two firmware generations. `am3` uses amps
in the `AM2` format, `am4` uses `AM4`. There is also `stationG_*`, a variant
of the same product.

**`*_ampCab.bin`** is 212 992 bytes (`0x34000`), that is, 17 records:

| Offset | Slot size | Contents (am4) |
|---|---|---|
| `0x00000` + n·`0x4000` | 16 KiB | **9 amps**: `AM4\0` + name (14 B) + floats (neural network). Factory: `1J120_CL`, `2J900_CL`, `3VAVE_TS`, `4RAT_OD`, `5VAVE_HTS`, `6VAVE_DS`, `7MS_GV2`, `8J900_HV`, `9MS_HIGAIN` |
| `0x24000` + n·`0x2000` | 8 KiB | **8 CABs**: `CAB\0` + name (14 B) + `02` + level + samples. Factory: `1AC-SeVin`, `2JVM_1960_57`, `3MESA_412`, `4WANGS112`, `5VOX_AC30`, `6HIFI_YAMAHA`, `7EGNL_412`, `8V30_MD421` |

The AMP header is the same as the MK-300's (`64 c8 3c 1e …` followed by 9
constant floats). An `.am2data` is therefore **one `AM2` amp record** and fits
a 16 KiB slot. On `am3`, the CAB takes about 6 204 bytes, close to the
BlackBox's and MK-300's 6 164 bytes (`CAB\0` + name + 2 bytes + 2048 × int24).

That answers "can more IRs be installed?": there are **9 amp slots and 8 IR
slots, fixed**. This validates the plan's "slots as a cache" design.

**`*_preset.bin`** is 758 bytes and starts with the magic `a5 5a`. Records are
about 21 bytes, with names that look obfuscated (`b9CTL`, `GENUL`…). It is not
the Tank Mini's raw 92-byte format; it must be compressed or encoded. ❓
Decode it through `structs.dart` or by cross-checking with the dump.

---

## 3. USB layer

| Item | Expected value | |
|---|---|---|
| Class | USB-MIDI 1.0 + USB Audio 2in/2out 44.1 kHz/24 bit; `snd-usb-audio` driver | 🟡 (IR Box, BlackBox and MK-300 are like this) |
| VID:PID | SINCO family `4353:xxxx` (IR Box = `4353:4B4D`) or Jieli `4c4a:xxxx` (BlackBox = `4c4a:c755`) | ❓ `lsusb` |
| Port name | `USB-Midi` (Tank Mini, IR Box) or `USB Composite Device` (MK-300, BlackBox) | ❓ `aconnect -l` |
| Linux | The ALSA sequencer coexists with PipeWire. `midir` uses ALSA seq | ✅ (BlackBox on Fedora/PipeWire) |

To find the pedal, match the port name and, if possible, also the VID:PID read
from sysfs. Generic Jieli pedals are all called "Composite".

---

## 4. Framing (raw frame, 8-bit)

```
off  0  1   2     3..5           6 .. 6+len-1   6+len
     00 59  cmd   len (u24 LE)   body           cs
```

- **Checksum:** `cs = (~sum(body)) & 0xFF`, without the 6 header bytes. An
  empty body gives `FF`. 🟡 (IR Box verified; Tank Mini and MK-300 give the
  same result.) The pedal **silently drops** frames with a bad checksum. ✅ (MK-300)
- **Commands:**

| cmd | Name | Use |
|---|---|---|
| `00` | ACK/status | reply to every write. `00 59 00 01 00 00 00 FF` = ok, status `01` = error |
| `11` | identity | returns name and firmware (e.g. `TANK MINI_009`, `IR-BOX_010`) |
| `12`,`13`,`17`,`1B`,`20` | queries | several get no reply, so use a short timeout |
| `21` | **ERASE** | ⛔ **FORBIDDEN.** The app has this command; our code must refuse to send it in every case |
| `22` | WRITE | `type(1) addr(u32 LE) len(u24 LE) data` |
| `23` | READ | request: `type addr len`. Reply: request echo + data |

### 4.1 7-bit packing for USB

The frame is read as one continuous bit stream, LSB first, and goes out in
7-bit pieces. It is not the "one MSB byte every 7 bytes" scheme common with
other manufacturers.

```
accum = 0; nbits = 0
for b in frame:
    accum |= b << nbits; nbits += 8
    while nbits >= 7: emit(accum & 0x7F); accum >>= 7; nbits -= 7
emit(accum & 0x7F)     # final flush byte, always emitted
```

`n` bytes become `n + n//7 + 1`. To unpack, use `n_raw = 7·len/8` and cut by
the header's `len`. **Never** strip trailing zeros, because the checksum can
be `00`.

Test vectors (IR Box and Cube Baby):

| Raw | On the wire |
|---|---|
| `00 59 11 00 00 00 FF` (identity query) | `F0 00 32 45 00 00 00 40 7F 00 F7` |
| `00 59 00 01 00 00 00 FF` (ACK ok) | `F0 00 32 01 08 00 00 00 00 7F 01 F7` |
| `00 59 22 09 00 00 05 09 00 00 80 01 00 00 01 6F` | `F0 00 32 09 49 00 00 40 02 09 00 00 00 18 00 00 00 01 5E 01 F7` |

The MK-300 doc describes the same frame "seen from the packed side" (fields
`CLS/CMD/FIELD/SUB`). It is the same thing. We always reason about the raw
frame.

Over **BLE** the Tank Mini sends the raw frame, without SysEx and without
packing: GATT `ae40`, TX `ae41`, RX `ae42`. The Tank-G does the same (see the
Bluetooth part of section 0).

---

## 5. Memory model (working hypothesis for the Tank-G)

Reads and writes address `type` + `addr u32`. In type `04` (DEV), the
**address's high nibble** picks the "family". The map below combines the
BlackBox (firmware analysis + reads), the Tank Mini (capture) and the MK-300
(capture):

| addr | Contents | Evidence | Tank-G |
|---|---|---|---|
| `0x0000_0000 + slot·92` | **Preset bank in flash.** 92 B (`0x5C`) records + a `PATCHEND` trailer (10 B). BlackBox: limit `0x1CCA` = 80×92+10 | BlackBox ✅, Tank Mini ✅ (save at `id·92`) | 🟡 36×92+10 = 3 322 B |
| `0x1000_0000 + off` | **Current patch (edit buffer)**, 92 B. Writing here changes the sound immediately, RAM only | BlackBox ✅, Tank Mini ✅ | 🟡 |
| `0x2000_0000` | 4 B status (active preset?) | BlackBox ✅ | ❓ |
| `0xD000_0000` | **Loopback** (1 byte on/off) | BlackBox ✅ (OFF/ON/OFF comparison) | 🟡 |
| `0xE000_0000 + id` | **Select preset** (write `01`) | Tank Mini ✅, IR Box ✅ | 🟡 |
| `0xA…` / `0xF…` | audition/refresh and commit/save | MK-300 ✅, IR Box ✅ | ❓ |

Presets can also be switched with Program Change `C0 <id>`. ✅ on the Tank
Mini; the Tank-G forum mentions PC for the 36 presets and CC for the modules.

### 5.1 Tank Mini commands translated into this model

The "`00 00 04` routing" the Tank Mini doc describes is actually
`len=…, type=04`. And `[addr] 00 00 10` is the u32 LE `0x100000xx`, that is,
family 1 at offset `xx`. It all fits:

```
Select preset      00 59 22 09 00 00 | 04 | id 00 00 E0 | 01 00 00 | 01 | cs
Block toggle       00 59 22 09 00 00 | 04 | TT 00 00 10 | 01 00 00 | 0/1 | cs
Change model       00 59 22 09 00 00 | 04 | MM 00 00 10 | 01 00 00 | id  | cs
Knob (u16 LE)      00 59 22 0A 00 00 | 04 | KK 00 00 10 | 02 00 00 | v 00 | cs
Read current patch 00 59 23 08 00 00 | 04 | 00 00 00 10 | 5C 00 00 | cs
Save to flash      00 59 22 64 00 00 | 04 | (id·92) u32 | 5C 00 00 | 92 B | cs
```

### 5.2 92-byte patch (Tank Mini layout, starting point)

```
0..5    chain order (block ids)
6..7    padding
8..13   on/off per block
14..19  model per block
20..91  knobs: 6 blocks × 12 B (6 × u16 LE, 0..100)
```

The Tank Mini has the FX, AMP, MOD, DLY, REV and CAB blocks. The Tank-G has IR
CAB, Noise Gate, Reverb, Delay, Mod and Amp. They are the same six, with the
Gate in place of FX. ❓ Still to confirm: order, addresses, where the **LED
color** and **IR Lv** live, and whether knobs go from 0 to 100.

---

## 6. IR and amp upload

Reproduced byte for byte on the MK-300 (jpfaria/mvave) and the BlackBox
(Bulrok/irload.py), from M-EFCS's own code (`utils/wave_utils.dart`,
`ui/modules/import_cab.dart`):

- **CAB record:** `CAB\0` + name (14 B) + `01` + "CAB Level" (default 50) +
  **2048 int24 LE mono samples at 44.1 kHz**, 6 164 B in total. Longer IRs
  are truncated at about 46 ms.
- **Resampling** (`WaveUtils.resample`): Kaiser-windowed sinc β=8.6, ±24
  taps, cutoff `min(1, dst/src)`, normalized by the sum of the weights,
  rounded and clamped to int24. Reproducing exactly that algorithm guarantees
  output identical to the editor's.
- **Input WAV:** the editor fails on Float32 ("device busy"). Convert to
  integer PCM first, which our app does on its own.
- **Sequence on the MK-300:** the blob in chunks of about 1015 B, with an ACK
  per chunk → *audition* (`0xA` = 2 for CAB) → *save* (`0xF`, slot) and a wait
  for the flash commit (a few seconds; commands sent during the commit can
  cancel it). The IR Box uses 173 B chunks. ❓ On the Tank-G, capture the
  chunk size, address and save command.
- **Amp (`.am2data`):** an `AM2\0` record of up to 16 KiB, probably written
  through the same path as CAB. ❓

---

## 7. Known pitfalls (from people who have been there)

1. **Stale buffer:** a read outside the map returns a well-formed reply, with
   a valid checksum and junk inside. Validate the expected length and check
   the content; a valid reply doesn't prove a valid read.
2. **`0x21` ERASE and the BLE OTA characteristics (`ae00`–`ae02`): never.**
3. Never write outside the known map. The pedal doesn't reject it.
4. A flash commit takes seconds. Send nothing until a read confirms it.
5. Writing a model resets that block's knobs to the model's defaults. Read
   back after writing.
6. On some pedals the display only rereads the names at boot.
7. **Borrowed state:** Loopback (and any USB audio mode) must go back to its
   previous state on every exit of the program, SIGINT included. On the
   MK-300, someone spent an hour with a muted rig because of this.
8. The official editor and the new app **can't be used at the same time.**
   M-EFCS keeps its own preset cache and overwrites the changes.
9. Before any flash write: **full backup**, and require explicit
   confirmation.

---

## 8. References

Pedals from the same family, all talking to M-EFCS:

- **Tank Mini**, full protocol (in Brazilian Portuguese): https://github.com/xxyurirangelxx/mvave-tank-mini-protocol
- **MK-300**, SysEx + byte-exact NAM/IR upload, M-EFCS decompiled with blutter: https://github.com/jpfaria/mvave (`docs/protocol.md`, `docs/learnings.md`)
- **BlackBox on Linux**, native control, `irload.py`, M-EFCS patch for Wine: https://github.com/Bulrok/mvave-blackbox-linux
- **BlackBox RE**, map of the DEV families, M-EFCS notes: https://github.com/jakino0/mvave-blackbox-reverse-engineering (`docs/mefcs-protocol.md`)
- **IR Box**, frame, checksum, packing, IR upload and pitfalls: https://github.com/marco-jardim/mvave-irbox-tools (`docs/usb.md`, `docs/protocol.md`)
- **Cube Baby**, a **Rust** crate over USB-MIDI: https://github.com/pferreir/cuvave-midi
- M-VAVE community index: https://github.com/cicloid/awesome-fm-1 ("Related M-VAVE projects" section)
- Wireshark filter for SysEx: `usbaudio.sysex.reassembled.data` (Mixxx wiki: https://github.com/mixxxdj/mixxx/wiki/Reverse-Engineering)
