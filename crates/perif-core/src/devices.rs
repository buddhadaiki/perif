// SPDX-License-Identifier: GPL-3.0-or-later

//! Devices perif recognises, matched by USB vendor and product ID.

use crate::hid::{HidEnumerator, HidError, HidInterface};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceKind {
    Mouse,
    Keyboard,
    Headset,
}

impl DeviceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            DeviceKind::Mouse => "mouse",
            DeviceKind::Keyboard => "keyboard",
            DeviceKind::Headset => "headset",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownDevice {
    pub vendor_id: u16,
    pub product_id: u16,
    pub kind: DeviceKind,
    pub brand: &'static str,
}

/// See `docs/devices.md` for how each entry was identified.
pub const KNOWN_DEVICES: &[KnownDevice] = &[
    KnownDevice {
        vendor_id: 0x24AE,
        product_id: 0x1244,
        kind: DeviceKind::Mouse,
        brand: "Rapoo",
    },
    KnownDevice {
        vendor_id: 0x3151,
        product_id: 0x502D,
        kind: DeviceKind::Keyboard,
        brand: "Akko",
    },
    KnownDevice {
        vendor_id: 0x291D,
        product_id: 0x385D,
        kind: DeviceKind::Headset,
        brand: "Mchose",
    },
];

pub fn identify(vendor_id: u16, product_id: u16) -> Option<&'static KnownDevice> {
    KNOWN_DEVICES
        .iter()
        .find(|d| d.vendor_id == vendor_id && d.product_id == product_id)
}

/// A recognised device together with every HID collection it exposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedDevice {
    pub known: &'static KnownDevice,
    pub interfaces: Vec<HidInterface>,
}

/// Enumerates HID collections and groups the recognised ones per device, in
/// the order of [`KNOWN_DEVICES`]. Interfaces are sorted by interface number,
/// usage page and usage.
pub fn detect(enumerator: &dyn HidEnumerator) -> Result<Vec<DetectedDevice>, HidError> {
    let all = enumerator.enumerate()?;
    Ok(KNOWN_DEVICES
        .iter()
        .filter_map(|known| {
            let mut interfaces: Vec<HidInterface> = all
                .iter()
                .filter(|i| i.vendor_id == known.vendor_id && i.product_id == known.product_id)
                .cloned()
                .collect();
            if interfaces.is_empty() {
                return None;
            }
            interfaces.sort_by_key(|i| (i.interface_number, i.usage_page, i.usage));
            Some(DetectedDevice { known, interfaces })
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeEnumerator(Vec<HidInterface>);

    impl HidEnumerator for FakeEnumerator {
        fn enumerate(&self) -> Result<Vec<HidInterface>, HidError> {
            Ok(self.0.clone())
        }
    }

    fn iface(vid: u16, pid: u16, number: i32, usage_page: u16, usage: u16) -> HidInterface {
        HidInterface {
            vendor_id: vid,
            product_id: pid,
            manufacturer: None,
            product: None,
            interface_number: number,
            usage_page,
            usage,
            path: format!("fake/{vid:04x}:{pid:04x}/{number}/{usage_page:04x}:{usage:04x}"),
        }
    }

    #[test]
    fn identifies_by_vid_and_pid() {
        assert_eq!(identify(0x24AE, 0x1244).unwrap().kind, DeviceKind::Mouse);
        assert!(identify(0x24AE, 0xFFFF).is_none());
        assert!(identify(0xFFFF, 0x1244).is_none());
    }

    #[test]
    fn groups_and_sorts_interfaces_and_skips_unknown_devices() {
        let fake = FakeEnumerator(vec![
            iface(0x3151, 0x502D, 2, 0xFFFF, 0x02),
            iface(0x24AE, 0x1244, 1, 0xFF00, 0x0E),
            iface(0x046D, 0xC08B, 0, 0x0001, 0x02),
            iface(0x24AE, 0x1244, 0, 0x0001, 0x02),
            iface(0x24AE, 0x1244, 1, 0x0001, 0x06),
        ]);

        let detected = detect(&fake).unwrap();

        assert_eq!(detected.len(), 2);
        assert_eq!(detected[0].known.brand, "Rapoo");
        let order: Vec<(i32, u16, u16)> = detected[0]
            .interfaces
            .iter()
            .map(|i| (i.interface_number, i.usage_page, i.usage))
            .collect();
        assert_eq!(
            order,
            [(0, 0x0001, 0x02), (1, 0x0001, 0x06), (1, 0xFF00, 0x0E)]
        );
        assert_eq!(detected[1].known.brand, "Akko");
    }

    #[test]
    fn absent_devices_are_not_reported() {
        assert!(detect(&FakeEnumerator(vec![])).unwrap().is_empty());
    }
}
