# Tank-G on Linux: mapping and app plan

Oct 17, 2026 · @Murilo Rocha

## What the official software does

The official editor (M-EFCS, Windows only) covers IRs, amp models, presets and
a fixed chain of six blocks. The new app has to do all of that.

**IR (cabinet)**
- Open an IR file (.wav)
- Pick the target slot (8 slots, see `PROTOCOL.md` §2.1)
- Write the IR to the pedal's slot
- IR Lv: the IR's output level
- Restore factory AMPs and IRs

**AM2Data (amp models)**
- Open an .am2data file, which is an external amp model (e.g. BOSS_Crunch_IR2)
- Pick and write it to an amp slot (9 slots)
- In other words: amps can be swapped, not just IRs

**Global**
- Bypass
- Loopback (PC audio returned through the USB interface)
- Restore factory presets

**Presets** (36, in the 1A…12C format)
- Select a preset
- Preset LED color
- Save to another slot (Save preset to)
- Swap two presets
- Import/export the current preset
- Import/export all presets

**Effect chain**

| Block | Parameters |
|---|---|
| IR CAB | IR slot |
| Noise Gate | level |
| Reverb | on/off, Mix, Decay |
| Delay | on/off, Time, Mix |
| Mod | on/off, Speed, FX (type) |
| Amp | on/off, Volume, Bass, Middle, Treble, Gain, Amp Type |

## Can more IRs be installed?

They can be replaced, not added. The pedal has **9 amp slots and 8 IR slots**
(confirmed by the factory banks inside M-EFCS), and the software only
overwrites one slot at a time.

The new app works around that by treating the slots as a cache of an
unlimited library on the PC:
- You keep hundreds of IRs and amps on the computer.
- Each preset points to its IR by file (hash), not by slot number.
- When loading a preset, the app checks whether the IR is already on the
  pedal. If not, it writes it to the least recently used slot and updates the
  preset.

Writing an IR takes time and wears the flash. That is why there is a "show"
mode, which pins the setlist's IRs on the pedal and swaps nothing live.

## What the new app adds

**IR and amp library**
- Drag a whole folder: the app indexes every .wav and .am2data
- Fuzzy search, tags (speaker, mic, cab size) and favorites
- Automatic conversion to the pedal's format (likely 2048 int24 mono samples at 44.1 kHz; to be confirmed by capture)
- Audition before writing: the IR is applied to a guitar DI on the PC itself, without spending flash
- Duplicate detection by hash

**Dense presets**
- Table view: 36 presets × every parameter, sortable, with multi-row editing
- Command palette (Ctrl+K): one search across presets, IRs, amps and parameters
- Unlimited preset library on the PC; the pedal gets 36 at a time
- Setlists: an ordered list of presets synced to the slots, with IRs and amps
- Side-by-side diff between two presets
- Undo/redo and per-preset version history
- One-click full backup of the pedal (presets, IRs and amps)

**Linux integration**
- CLI (`tankg preset 5`, `tankg ir load ~/irs/v30.wav`) for Hyprland binds
- MIDI monitor for debugging and mapping

## Captures needed

See the detailed checklist and the method (Wine + usbmon, no VM) in
[`CAPTURES.md`](CAPTURES.md).
