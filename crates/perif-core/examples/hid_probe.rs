// SPDX-License-Identifier: GPL-3.0-or-later

//! Read-only reconnaissance helper. For every HID collection of one device it
//! prints the report descriptor and then listens for input reports. It never
//! sends anything to the device.
//!
//! Usage: `cargo run -p perif-core --example hid_probe -- <vid-hex> <pid-hex> [listen-seconds]`

use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [vid, pid, rest @ ..] = args.as_slice() else {
        return Err("usage: hid_probe <vid-hex> <pid-hex> [listen-seconds]".into());
    };
    let vid = u16::from_str_radix(vid, 16)?;
    let pid = u16::from_str_radix(pid, 16)?;
    let listen = Duration::from_secs(rest.first().map_or(Ok(0), |s| s.parse())?);

    let api = hidapi::HidApi::new()?;
    let mut infos: Vec<_> = api
        .device_list()
        .filter(|d| d.vendor_id() == vid && d.product_id() == pid)
        .collect();
    infos.sort_by_key(|d| (d.interface_number(), d.usage_page(), d.usage()));

    for info in infos {
        println!(
            "== iface {} usage {:04X}:{:04X}\n   path {}",
            info.interface_number(),
            info.usage_page(),
            info.usage(),
            info.path().to_string_lossy()
        );
        let device = match api.open_path(info.path()) {
            Ok(device) => device,
            Err(e) => {
                println!("   open failed: {e}");
                continue;
            }
        };

        let mut descriptor = [0u8; 4096];
        match device.get_report_descriptor(&mut descriptor) {
            Ok(len) => println!("   descriptor ({len} bytes): {}", hex(&descriptor[..len])),
            Err(e) => println!("   descriptor unavailable: {e}"),
        }

        let deadline = Instant::now() + listen;
        let mut buf = [0u8; 256];
        while Instant::now() < deadline {
            match device.read_timeout(&mut buf, 250) {
                Ok(0) => {}
                Ok(len) => println!("   input ({len} bytes): {}", hex(&buf[..len])),
                Err(e) => {
                    println!("   read failed: {e}");
                    break;
                }
            }
        }
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}
