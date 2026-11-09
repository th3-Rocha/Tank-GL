# Captures: how to do them and what to capture

Goal: confirm on the real Tank-G the hypotheses in `PROTOCOL.md`. There are
two complementary fronts: **dynamic capture** of the USB traffic and **static
analysis** of the editor's Dart code.

## 1. Running M-EFCS on Linux (no VM)

The Windows editor runs under Wine. The only problem is the Bluetooth plugin,
which aborts at startup because Wine doesn't implement
`Windows.Devices.Radios`. The fix is to change the first byte of
`universal_ble_plugin.dll` (its only export, the plugin registrar) to `C3`
(`ret`), which disables the plugin. Source and script: Bulrok/mvave-blackbox-linux
(`tools/patch_mefcs.py`).

```bash
export WINEPREFIX=~/.local/share/wineprefixes/mvave
wineboot -i
winetricks dxvk                                   # optional
unzip ~/Downloads/M-EFCS.zip -d "$WINEPREFIX/drive_c/"
# Patch: offset 0xBB0 of universal_ble_plugin.dll, 40 53 48 83 → C3 53 48 83
# Bulrok's script checks the hash of their build. Ours (2026-10-10) may be
# different, so check the bytes at the offset before writing. Keep the .orig.
wine "$WINEPREFIX/drive_c/M-EFCS/m_efcs.exe"
```

If Wine doesn't work: a Windows VM with USB passthrough. The rest of the
method stays the same, because usbmon runs on the host.

## 2. Dynamic capture with usbmon

```bash
lsusb                                  # note the Tank-G's VID:PID and bus
aconnect -l                            # note the MIDI port name
sudo modprobe usbmon
sudo tshark -i usbmon<BUS> -w captures/<action>.pcapng
# analysis: Wireshark with the filter  usbaudio.sysex.reassembled.data
```

You can also see only what the pedal sends with `aseqdump -p <port>`. For the
host→pedal direction, use usbmon.

Rules:
- **One action per file.** Name = action (`knob-reverb-mix-0-50-100.pcapng`).
- Wait about 5 s idle before each action, to separate the editor's polling
  from the action itself.
- Note the firmware (query `11`) and the M-EFCS version in
  `captures/README.md`.
- `.pcapng` files stay out of git (`.gitignore`). Decodes and notes go to
  `docs/`.

## 3. Checklist (from the plan, with what each item answers)

- [ ] **Open the editor with the pedal connected.** Shows the handshake, the
      identity query and the initial dumps. That's the memory map for free.
- [ ] **Turn each knob 0 → middle → max.** Shows the knob's address, width
      (u8/u16) and scale.
- [ ] **Turn each block and Bypass on and off.** Shows the toggle addresses
      and whether bypass is global or per preset.
- [ ] **Switch preset, change the color, Save preset to, Swap.** Shows the
      select (`0xE…`?), where the color lives, the flash save (`id·92`?) and
      whether swap is its own command or two saves.
- [ ] **Export current preset and Export all presets.** Shows the exported
      file format and the bank dump (family 0?).
- [ ] **Write an IR to a slot.** The most important one: chunk size, address,
      audition and save. Compare the blob with the `CAB\0` record that
      `irload.py` builds from the same WAV.
- [ ] **Write an AM2Data.** Shows whether it uses the same path as CAB and the
      size of the `AM2\0` record.
- [ ] **Turn Loopback on and off.** Confirms `0xD000_0000`.
- [ ] **IR Lv.** Shows whether it is global or per preset, and the address.
- [ ] **Factory reset (presets, amps/IRs).** Do it **last**, and only after a
      backup. Shows whether the editor resends the bundled factory `.bin`
      files or whether it is a pedal command.
- [ ] **Switch preset from the footswitch** (without touching the editor).
      Shows whether the pedal notifies the host or the editor polls.

## 4. Static analysis (in parallel)

- **The Windows `data/app.so`** is Dart AOT x86-64. Strings and class names
  can be extracted (that is how we found `tank_g/structs.dart`). Recovering
  code is harder.
- **The M-EFCS Android APK + [blutter](https://github.com/worawit/blutter)**
  recovers the Dart code (arm64): `tank_g/structs.dart`, `effect_params.dart`,
  `sysex_codec.dart`, `wave_utils.dart`. That is the path jpfaria/mvave used
  to nail the upload byte for byte. Dart 3.11.x is supported.
- **`resources/bin/tankG/*.bin`** carries the factory banks. They serve as a
  reference for the AMP/CAB formats and as an offline "restore factory".
  They belong to the manufacturer: **don't commit them**, read them at
  runtime from an M-EFCS the user has.

## 5. What we need from the pedal (first plug)

Read-only, no writes:

1. `lsusb -v -d <vid:pid>` and `aconnect -l`
2. Query `11` (identity/firmware)
3. Read the current patch (`04 / 0x1000_0000 / 0x5C`) and check it against
   what the display shows
4. Read the bank (`04 / 0x0000_0000`) in chunks of up to 1000 B, for a backup
   and to validate the size
