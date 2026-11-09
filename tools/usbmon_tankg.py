#!/usr/bin/env python3
"""Captures the Tank-G's USB-MIDI traffic through usbmon and decodes the frames.

Usage (needs to read /dev/usbmonN; see docs/CAPTURES.md):

    python3 -I tools/usbmon_tankg.py captures/action.log

Each line: time, direction (PC→ pedal / ← pedal), command and body. Repeated
status reads (the editor's polling) are summarized so they don't drown the action.
"""

import ctypes
import fcntl
import os
import struct
import sys
import time

VID, PID = "4c4a", "c755"
MON_IOCX_GETX = 0x4018920A  # _IOW(0x92, 10, struct mon_bin_get), 64 bits
HDR = struct.Struct("<QBBBBHbbqiiII8siiII")  # struct mon_bin_hdr, 64 B
CMDS = {0x00: "ACK", 0x11: "IDENT", 0x21: "ERASE", 0x22: "WRITE", 0x23: "READ"}
# Data bytes per USB-MIDI packet, by Code Index Number.
CIN_LEN = {0x4: 3, 0x5: 1, 0x6: 2, 0x7: 3, 0x8: 3, 0x9: 3, 0xA: 3, 0xB: 3, 0xC: 2, 0xD: 2, 0xE: 3, 0xF: 1}


def find_device():
    for d in os.listdir("/sys/bus/usb/devices"):
        base = f"/sys/bus/usb/devices/{d}"
        try:
            if open(f"{base}/idVendor").read().strip() == VID and open(f"{base}/idProduct").read().strip() == PID:
                return int(open(f"{base}/busnum").read()), int(open(f"{base}/devnum").read())
        except OSError:
            continue
    sys.exit("Tank-G not found on USB")


def unpack7(wire):
    out, acc, nbits = [], 0, 0
    for b in wire:
        acc |= (b & 0x7F) << nbits
        nbits += 7
        if nbits >= 8:
            out.append(acc & 0xFF)
            acc >>= 8
            nbits -= 8
    return bytes(out)


def describe(sysex):
    """Raw frame `00 59 cmd len(u24) body cs` from inside a SysEx."""
    raw = unpack7(sysex[1:-1])
    if len(raw) < 7 or raw[:2] != b"\x00\x59":
        return "sysex " + sysex.hex(" ")
    cmd = raw[2]
    n = int.from_bytes(raw[3:6], "little")
    body = raw[6 : 6 + n]
    cs = raw[6 + n] if len(raw) > 6 + n else None
    ok = cs == (~sum(body)) & 0xFF
    name = CMDS.get(cmd, f"cmd {cmd:02x}")
    txt = f"{name:<5} len={n:<4}"
    if cmd in (0x21, 0x22, 0x23) and len(body) >= 8:
        typ, addr, ln = body[0], int.from_bytes(body[1:5], "little"), int.from_bytes(body[5:8], "little")
        txt += f" type={typ:02x} addr=0x{addr:08x} n={ln}"
        data = body[8:]
        if data:
            txt += f" data[{len(data)}]={data.hex(' ')}"
    else:
        txt += f" body={body.hex(' ')}"
    if not ok:
        txt += "  (BAD checksum)"
    return txt


def main():
    out = open(sys.argv[1], "a", buffering=1) if len(sys.argv) > 1 else None
    bus, dev = find_device()
    fd = os.open(f"/dev/usbmon{bus}", os.O_RDONLY)
    hdr = ctypes.create_string_buffer(HDR.size)
    data = ctypes.create_string_buffer(65536)
    arg = struct.pack("<QQQ", ctypes.addressof(hdr), ctypes.addressof(data), len(data))
    pending = {True: bytearray(), False: bytearray()}
    t0 = time.monotonic()
    last_poll = {}

    def emit(line):
        print(line, flush=True)
        if out:
            out.write(line + "\n")

    emit(f"# Tank-G capture bus {bus} dev {dev}, {time.strftime('%F %T')}")
    while True:
        fcntl.ioctl(fd, MON_IOCX_GETX, arg)
        f = HDR.unpack(hdr.raw)
        typ, xfer, ep, devnum, len_cap = chr(f[1]), f[2], f[3], f[4], f[12]
        if devnum != dev or xfer not in (2, 3):  # bulk or interrupt
            continue
        to_dev = not (ep & 0x80)
        # OUT: data comes in the submit (S); IN: in the complete (C).
        if (to_dev and typ != "S") or (not to_dev and typ != "C") or len_cap == 0:
            continue
        payload = data.raw[:len_cap]
        for i in range(0, len(payload) - 3, 4):
            cin = payload[i] & 0x0F
            if payload[i : i + 4] == b"\x00\x00\x00\x00":
                continue
            msg = payload[i + 1 : i + 1 + CIN_LEN.get(cin, 0)]
            buf = pending[to_dev]
            if cin in (0x4, 0x5, 0x6, 0x7) or (buf and cin == 0xF):
                buf.extend(msg)
                if msg and msg[-1] == 0xF7:
                    sysex = bytes(buf)
                    buf.clear()
                    line = describe(sysex)
                    # summarize polling: the same repeated read only shows again after 5 s
                    key = (to_dev, line)
                    now = time.monotonic()
                    if ("READ" in line or "ACK" in line) and now - last_poll.get(key, -99) < 5:
                        continue
                    last_poll[key] = now
                    emit(f"{now - t0:9.3f} {'PC→' if to_dev else '  ←'} {line}")
            elif msg:
                emit(f"{time.monotonic() - t0:9.3f} {'PC→' if to_dev else '  ←'} midi {msg.hex(' ')}")


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
