# Identified devices

Captured on 2026-11-18, Windows 11, with `perif list` (hidapi 2.6.7,
`windows-native` backend). USB revision and serial strings were read from the
Windows PnP properties of the same devices.

## Summary

| Kind     | Brand  | Model                           | VID:PID     | Manufacturer string       | Product string        | bcdDevice | Connection                |
|----------|--------|---------------------------------|-------------|---------------------------|-----------------------|-----------|---------------------------|
| Mouse    | Rapoo  | Unknown (to be determined)      | `24AE:1244` | `Rapoo`                   | `Rapoo Gaming Device` | `0100`    | 2.4G dongle (see notes)   |
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

- The USB serial string is `DONGLE_RTL_B-CUT_20240408`, so this VID:PID is a
  2.4G receiver, not the mouse itself over a cable. The string suggests a
  Realtek-based dongle; that is an inference, not a confirmed fact.
- The mouse model is not exposed in any USB string.
- The four vendor-defined collections on interface 1 are the candidates for
  the configuration channel. Which one is used is not yet known.

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
