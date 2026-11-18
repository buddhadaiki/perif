// SPDX-License-Identifier: GPL-3.0-or-later

use std::process::ExitCode;

use perif_core::devices::{self, identify};
use perif_core::hid::{HidApiEnumerator, HidEnumerator, HidInterface};

const USAGE: &str = "\
Usage:
  perif list          Show recognised peripherals and their HID interfaces
  perif list --all    Show every HID interface on the system
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();

    let result = match args.as_slice() {
        ["list"] => list(false),
        ["list", "--all"] => list(true),
        _ => {
            eprint!("{USAGE}");
            return ExitCode::from(2);
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("perif: {e}");
            ExitCode::FAILURE
        }
    }
}

fn list(all: bool) -> Result<(), Box<dyn std::error::Error>> {
    let enumerator = HidApiEnumerator::new()?;

    let rows: Vec<(&str, HidInterface)> = if all {
        let mut interfaces = enumerator.enumerate()?;
        interfaces.sort_by_key(|i| {
            (
                i.vendor_id,
                i.product_id,
                i.interface_number,
                i.usage_page,
                i.usage,
            )
        });
        interfaces
            .into_iter()
            .map(|i| {
                let kind = identify(i.vendor_id, i.product_id).map_or("-", |d| d.kind.as_str());
                (kind, i)
            })
            .collect()
    } else {
        devices::detect(&enumerator)?
            .into_iter()
            .flat_map(|d| {
                let kind = d.known.kind.as_str();
                d.interfaces.into_iter().map(move |i| (kind, i))
            })
            .collect()
    };

    if rows.is_empty() {
        println!("No recognised peripherals found. Try `perif list --all`.");
        return Ok(());
    }
    print_table(&rows);
    Ok(())
}

fn print_table(rows: &[(&str, HidInterface)]) {
    let text = |s: &Option<String>| s.clone().unwrap_or_else(|| "-".to_owned());
    let width = |header: &str, field: fn(&HidInterface) -> &Option<String>| {
        rows.iter()
            .map(|(_, i)| text(field(i)).chars().count())
            .chain([header.len()])
            .max()
            .unwrap_or(0)
    };
    let mw = width("MANUFACTURER", |i| &i.manufacturer);
    let pw = width("PRODUCT", |i| &i.product);

    println!(
        "{:<8}  {:<4}  {:<4}  {:<mw$}  {:<pw$}  {:>5}  {:<10}  USAGE",
        "KIND", "VID", "PID", "MANUFACTURER", "PRODUCT", "IFACE", "USAGE PAGE"
    );
    for (kind, i) in rows {
        println!(
            "{:<8}  {:04X}  {:04X}  {:<mw$}  {:<pw$}  {:>5}  0x{:04X}      0x{:04X}",
            kind,
            i.vendor_id,
            i.product_id,
            text(&i.manufacturer),
            text(&i.product),
            i.interface_number,
            i.usage_page,
            i.usage,
        );
    }
}
