# Rapoo VT3 Pro Max: configuration protocol

Status: **draft**. Everything below comes from static analysis of the vendor
software and from reading descriptors. Nothing has been sent to the mouse yet,
so no command is confirmed on this hardware. Each statement is tagged with
where it came from:

- **[desc]** read from the device's HID report descriptors with
  `cargo run -p perif-core --example hid_probe -- 24ae 1244`.
- **[obs]** input report observed from the device without sending anything.
- **[bin]** found in the vendor binary, `RapooGameDev.exe` 1.6.29 (32-bit,
  image base `0x400000`); addresses are virtual addresses in that build.
- **[3p]** stated by a third-party project and not independently checked here.

## Device

| | |
|---|---|
| Model | Rapoo VT3 Pro Max, named `VT3PRO MAX HSDM` by the vendor software |
| 2.4G dongle | `24AE:1244`, USB serial string `DONGLE_RTL_B-CUT_20240408` |
| Wired | presumably `24AE:1215` (PID stored in the vendor profile); not checked |
| Vendor software | RapooGameDev 1.6.29 (Qt 5) |
| Vendor driver class | `FDeviceVT3PROMaxHSDM_Wireless_BCut` (dongle), `FDeviceVT3PROMaxHSDM_BCut` (wired) **[bin]** |

### Why other Rapoo documentation applies

**[bin]** The RTTI class hierarchy in the vendor binary is:

```
FDeviceVT3PROMaxHSDM_Wireless_BCut
  <- FDeviceVT9AirHSDM_Wireless <- FDeviceVT9AirHSDM <- FDeviceVT9PRO
  <- FDeviceVT3S <- FDeviceV30W <- FDeviceVT960_Nordic_Wireless
  <- KDeviceVT950_Nordic_Wireless <- KDeviceVT950_Nordic <- KBaseDevice <- QObject
```

`FDeviceVT3PRO_HSDM_Wireless_BCut` (VT3 PRO) and
`FDeviceVT9PROHSDM_Wireless_BCut` (VT9 PRO) have the same parent, and the
virtual tables of all three classes are identical in all 87 slots. The three
models therefore run exactly the same driver code; they differ only in data set
by their constructors. Compared with the parent class, only slot 3 differs.

This matters because two open projects document that code path:

- [D3m0nZOnFire/mousectl](https://github.com/D3m0nZOnFire/mousectl) (MIT),
  driver `rapoo_vt3pro`, reverse engineered from the same RapooGameDev 1.6.29
  and reported by its author as checked on a VT3 PRO (`24AE:1231`), including
  writes.
- [OpenMouse mouse-protocol](https://github.com/OpenMouse-Project/mouse-protocol)
  (AGPL-3.0), with a capture from a VT9 Pro (`24AE:1205` / `24AE:4405`).

No code from either project is used in perif.

## Transport

All on USB interface 1 of the dongle.

| Collection | Report ID | Direction | Data length | Role |
|---|---|---|---|---|
| `0xFF00:0x000E` | `0xBA` | output and input | 31 bytes | Command channel **[desc]** |
| `0xFF00:0x0002` | `0xBB` | input | 6 bytes | Unsolicited status, about every 3 s **[desc][obs]** |
| `0xFF00:0x0002` | `0xBC` | input and output | 9 bytes | Unknown **[desc]** |
| `0xFF0B:0x0104` | `0x2A`..`0x2D` | input, output, feature | up to 60 bytes | Not used by perif, see Hazards **[desc]** |

**[bin]** A command is exchanged as follows (`0x01129480`, virtual slot 84 of
the wireless class):

1. Build a 32-byte zeroed buffer holding the frame.
2. Send it with `HidD_SetOutputReport`, report ID `0xBA`
   (`0x01128B80` -> `0x00FC6A30` -> `0x00FBD080`). The vendor code passes its
   32-byte buffer plus the report ID; the descriptor allows 31 data bytes, so
   the report on the wire is the ID followed by the first 31 bytes. How the
   wrapper trims the last byte has not been analysed.
3. Sleep 50 ms.
4. Up to three times: fetch the answer with `HidD_GetInputReport`, report ID
   `0xBA` (`0x01182B30` -> `0x00FC69A0` -> `0x00FBCA40`), and look at the first
   byte after the report ID. Sleep 50 ms after each attempt.
5. If none of the three attempts returned status `0x01`, a fallback routine at
   `0x01128D60` runs. It has not been analysed.

The answer is **not** delivered as an interrupt input report; it must be
requested with GET_REPORT(Input).

### Request frame

**[bin]** Built by `0x011825D0`. Offsets are within the report data, so the
report ID `0xBA` precedes byte 0 on the wire.

| Offset | Size | Field |
|---:|---:|---|
| 0 | 1 | Connection byte, copied from the device object (`this+0x1BC`). The dongle class constructor (`0x0110FCAC`) sets `0xA5`; the wired class constructor sets `0xFF`. |
| 1 | 1 | Command: `0xA4` read, `0xA5` write |
| 2 | 1 | Length of the block in bytes |
| 3 | 4 | Address, little endian |
| 7 | n | Data (writes only) |
| | | Zero padding up to 31 bytes |

There is no checksum and no handshake in this code path.

### Answer

**[bin]** `0x01182B30` classifies the first byte after the report ID:

| Status | Meaning in the vendor code |
|---|---|
| `0x00` | No answer yet (retry) |
| `0x01` | Answer ready; the buffer is copied out |
| `0x02` | Busy (retry) |

**[bin]** The DPI reader (`0x01177D10`) takes the block from offset 4 of the
answer, counted after the report ID: bytes 0..3 are status and header, the
payload starts at byte 4.

**[3p]** mousectl and OpenMouse both report that the link drops frames, and
that an answer with status `0x01` can be the stale answer to the previous
command. This has not been measured here.

## Settings memory

Settings are read and written as blocks of an EEPROM-like address space.

| Address | Length used by vendor | Content | Source |
|---|---:|---|---|
| `0x0888` | 24 (`0x18`) | DPI X table | **[bin]** `0x01177D10` reads it with `A4`, length `0x18` |
| `0x08C8` | 24 (`0x18`) | DPI Y table | **[bin]** same function |
| `0x0898` | 4 | Active DPI stage in byte 0 | **[bin]** `0x01177FE0` rejects a stage greater than 7, reads the block (length 4), and builds a write frame (`A5`, length 4) with byte 0 replaced by the requested stage and bytes 1..3 copied from what it read |

**[3p]** Layout of the DPI tables per mousectl and OpenMouse: seven
little-endian `u16` DPI values, then a stage-count byte at offset 14. They read
16 bytes; the vendor binary reads 24. The meaning of bytes 15..23 is unknown.

**[3p]** Other blocks documented by those projects and not yet checked against
the binary here: `0x0880` polling rate, `0x0884` lift-off and motion sync,
`0x08C0` debounce, sleep and correction flags, `0x08C4` sensor angle.

### Frames perif will send to read the DPI

Derived only from the **[bin]** items above. Full 32-byte output report: the
report ID, then the 31 data bytes.

| Purpose | Bytes |
|---|---|
| Read active stage | `BA A5 A4 04 98 08 00 00` + 24 x `00` |
| Read DPI X table | `BA A5 A4 18 88 08 00 00` + 24 x `00` |
| Read DPI Y table | `BA A5 A4 18 C8 08 00 00` + 24 x `00` |

## Status report `0xBB`

**[obs]** Observed while idle: `bb b0 81 20 03 01 64`.

**[3p]** Per zbndev/rapoo_battery: byte 5 is the charging state (`01`
discharging, `02` charging, `00` in transition) and byte 6 the battery
percentage. The observation is consistent with a discharging mouse at 100 %.
The meaning of bytes 1..4 is unknown.

## Hazards

- **[3p]** OpenMouse reports that reading **feature report `0x2B`** on the
  `0xFF0B:0x0104` collection of a VT9 Pro disconnects the mouse until it is
  re-plugged. perif does not open that collection for anything other than
  reading its descriptor.
- **[bin]** The vendor package ships `RTKHIDKit.dll` (Realtek), whose exports
  include pairing, factory reset and firmware update (`RTKBTCtrlFactoryReset`,
  `RTKCFUAddImage`, ...), plus `FirmwareExe.exe` and `Qt_FwUpdates.dll`. Which
  HID collection those use is not established; `0xFF0B:0x0104` is the likely
  candidate. perif sends nothing there.

## Unknown

- The fallback exchange routine at `0x01128D60`.
- How the vendor software writes the DPI tables (not yet analysed), and which
  DPI values it accepts for this model.
- Bytes 15..23 of the DPI blocks, bytes 1..3 of the active-stage block.
- Whether settings persist across a power cycle without a separate commit
  command. One third-party project mentions a commit command `0xA1`; nothing in
  the code analysed so far uses it.
- Reports `0xBC` and `0x2A`..`0x2D`.
