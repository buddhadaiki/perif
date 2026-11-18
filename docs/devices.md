# Identified devices

Captured on 2026-11-18, Windows 11, with `perif list` (hidapi 2.6.7,
`windows-native` backend). USB revision and serial strings were read from the
Windows PnP properties of the same devices.

## Summary

| Kind     | Brand  | Model                           | VID:PID     | Manufacturer string       | Product string        | bcdDevice | Connection                |
|----------|--------|---------------------------------|-------------|---------------------------|-----------------------|-----------|---------------------------|
| Mouse    | Rapoo  | VT3 Pro Max (`VT3PRO MAX HSDM`) | `24AE:1244` | `Rapoo`                   | `Rapoo Gaming Device` | `0100`    | 2.4G dongle (see notes)   |
| Keyboard | Akko   | Unknown (to be determined)      | `3151:502D` | (empty)                   | `Akko Keyboard`       | `0502`    | Not determined            |
| Headset  | Mchose | V9 Pro (from the product string)| `291D:385D` | `C-Media Electronics Inc` | `MCHOSE V9 PRO`       | `0012`    | Not determined            |

## HID interfaces

`IFACE` is the USB interface number. On Windows every top-level collection of
an interface is listed as a separate entry, so one interface can appear on
several rows.

### Rapoo mouse, `24AE:1244`

| IFACE | Usage page | Usage    | Meaning                                |
|------:|------------|----------|----------------------------------------|
| 0     | `0x0001`   | `0x0002` | Generic Desktop / Mouse                |
| 1     | `0x0001`   | `0x0006` | Generic Desktop / Keyboard             |
| 1     | `0x0001`   | `0x0080` | Generic Desktop / System Control       |
| 1     | `0x000C`   | `0x0001` | Consumer / Consumer Control            |
| 1     | `0xFF00`   | `0x0002` | Vendor-defined                         |
| 1     | `0xFF00`   | `0x0002` | Vendor-defined (second collection)     |
| 1     | `0xFF00`   | `0x000E` | Vendor-defined                         |
| 1     | `0xFF0B`   | `0x0104` | Vendor-defined                         |

Notes:

- **Model.** The official software stores its settings for this mouse under
  the device name `VT3PRO MAX HSDM`
  (`%LOCALAPPDATA%\RapooGameDev\config\VT3PRO MAX HSDM\`). The model is not
  exposed in any USB string. Two third-party projects independently list
  `24AE:1244` as the VT3 Pro Max
  ([zbndev/rapoo_battery](https://github.com/zbndev/rapoo_battery),
  [kangstudy997-byte/rapoo-linux-driver](https://github.com/kangstudy997-byte/rapoo-linux-driver)).
  What the `HSDM` suffix means is unknown.
- **Connection.** The USB serial string is `DONGLE_RTL_B-CUT_20240408`, so
  `24AE:1244` is the 2.4G receiver, not the mouse over a cable. The official
  software's saved profile records PID `0x1215` for the same device, which is
  presumably the wired PID; this has not been checked by plugging the cable.
- **Official software.** RapooGameDev 1.6.29 (Rapoo, Inc.), a Qt 5 desktop
  application installed in `C:\Program Files (x86)\Rapoo\RapooGameDev`. It is
  not Electron. It ships `RTKHIDKit.dll` (Realtek) and a separate
  `FirmwareExe.exe` / `Qt_FwUpdates.dll` for firmware updates.
- **Not the same family as Rapoo's newer mice.** The newer Nordic-based
  models (driven by "A HUB" and `hub.rapoo.cn`) use report IDs 6, 7 and 8.
  This dongle has none of those; see the report table below.

Reports on interface 1, read from the HID report descriptors with
`cargo run -p perif-core --example hid_probe -- 24ae 1244`:

| Usage page:usage | Report ID | Input   | Output  | Feature | Notes |
|------------------|-----------|---------|---------|---------|-------|
| `0xFF00:0x000E`  | `0xBA`    | 31 B    | 31 B    | none    | Candidate command channel. |
| `0xFF00:0x0002`  | `0xBB`    | 6 B     | 6 B     | none    | Sent unsolicited about every 3 s. Observed `bb b0 81 20 03 01 64`, which matches the battery report documented by zbndev/rapoo_battery (byte 5 charging state, byte 6 percent). |
| `0xFF00:0x0002`  | `0xBC`    | 9 B     | 9 B     | none    | Nothing observed while idle. |
| `0xFF0B:0x0104`  | `0x2A`    | 60 B    | 60 B    | 60 B    | OpenMouse reads a status block from the feature report on the VT9 Pro. Not read here. |
| `0xFF0B:0x0104`  | `0x2B`    | none    | none    | 60 B    | Purpose unknown. |
| `0xFF0B:0x0104`  | `0x2C`    | 4 x 32 bit | none | none    | Purpose unknown. |
| `0xFF0B:0x0104`  | `0x2D`    | 4 x 32 bit | 4 x 32 bit | none | Purpose unknown. |

The purpose of the `0xFF0B` collection is not established. The official
package bundles a Realtek HID kit and a firmware updater, so it may be a
firmware channel.

Search for prior work on `24AE:1244` (2026-11-18):

| Project | Covers this device? |
|---------|---------------------|
| libratbag | No. No Rapoo device file among its 126 device files. |
| OpenRGB | No match for the vendor ID in the GitHub mirror. |
| [OpenMouse](https://github.com/OpenMouse-Project/mouse-protocol) (AGPL-3.0) | Not this PID. It documents the Rapoo VT9 Pro (`24AE:1205` / `24AE:4405`), which exposes the same collections and the same `0xBA` / `0xBB` reports, with a capture from real hardware. Read-only driver. |
| [D3m0nZOnFire/mousectl](https://github.com/D3m0nZOnFire/mousectl) (MIT) | Not this PID. Its `rapoo_vt3pro` driver targets the VT3 PRO (`24AE:1231` / `24AE:4431`) and was reverse engineered from RapooGameDev 1.6.29, the same version installed here. Reads and writes over `0xBA`. |
| [kangstudy997-byte/rapoo-linux-driver](https://github.com/kangstudy997-byte/rapoo-linux-driver) (MIT) | Names `24AE:1244` and VT3PRO MAX, report IDs `0xBA`/`0xBB`. Single commit, no stars, unverified. |
| [zbndev/rapoo_battery](https://github.com/zbndev/rapoo_battery) (GPL-2.0) | Lists `24AE:1244`. Battery report only; its `0xBB` layout matches what this dongle sends. |
| [pedro3z0/rapoo-software-linux](https://github.com/pedro3z0/rapoo-software-linux) (MIT) | No. Newer Nordic family (report IDs 6/7/8). Its settings memory layout lines up with the two projects above. |
| [Nuitfanee/ClickSync](https://github.com/Nuitfanee/ClickSync) (GPL-2.0), [Iris-0109/rapoo-tray](https://github.com/Iris-0109/rapoo-tray) (MIT) | No. Newer family only. |

Hazard reported by OpenMouse for the VT9 Pro: reading **feature report
`0x2B`** on the `0xFF0B:0x0104` collection disconnects the mouse until it is
re-plugged. perif never reads or writes that collection.

### Akko keyboard, `3151:502D`

| IFACE | Usage page | Usage    | Meaning                          |
|------:|------------|----------|----------------------------------|
| 0     | `0x0001`   | `0x0006` | Generic Desktop / Keyboard       |
| 1     | `0x0001`   | `0x0002` | Generic Desktop / Mouse          |
| 1     | `0x0001`   | `0x0006` | Generic Desktop / Keyboard       |
| 1     | `0x0001`   | `0x0080` | Generic Desktop / System Control |
| 1     | `0x000C`   | `0x0001` | Consumer / Consumer Control      |
| 1     | `0xFFFF`   | `0x0001` | Vendor-defined                   |
| 2     | `0xFFFF`   | `0x0002` | Vendor-defined                   |

Notes:

- The model is not exposed in any USB string.
- There is no `0xFF60`/`0x61` collection, which is the raw HID usage that
  VIA/QMK firmware normally exposes. This points away from VIA and towards the
  Akko Cloud Driver family, but it has not been confirmed.

### Mchose headset, `291D:385D`

| IFACE | Usage page | Usage    | Meaning                     |
|------:|------------|----------|-----------------------------|
| 0     | `0x000B`   | `0x0001` | Telephony / Phone           |
| 0     | `0x000C`   | `0x0001` | Consumer / Consumer Control |
| 0     | `0xFF21`   | `0x0001` | Vendor-defined              |
| 0     | `0xFF22`   | `0x0001` | Vendor-defined              |
| 0     | `0xFF82`   | `0x0001` | Vendor-defined              |
| 0     | `0xFF90`   | `0x0001` | Vendor-defined              |

Notes:

- Model confirmed as V9 Pro by the USB product string.
- The manufacturer string is the audio chip vendor (C-Media), not Mchose.
  Interface 1 of the same device is the USB audio function and is not HID.
- The USB serial string is the placeholder `0123456789AB`.
