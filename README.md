# perif

One open-source configurator for peripherals (mice, keyboards, headsets), so
you do not need a different vendor app for each of them.

**Status: spike.** This repository currently proves only the risky part,
talking to the hardware. There is no GUI, and only a handful of devices are
recognised. The name is provisional.

## Layout

- `crates/perif-core`: library that talks to devices.
- `crates/perif-cli`: thin command-line frontend (`perif`).
- `docs/devices.md`: devices identified so far.
- `docs/protocols/`: protocol notes, one file per device.

## Building

Windows is the only tested platform for now.

```
cargo build
cargo run -p perif-cli -- list
```

## Safety

perif only sends packets that were observed coming from the vendor's own
software or found in its driver code. It never sends firmware, DFU or
bootloader commands.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
