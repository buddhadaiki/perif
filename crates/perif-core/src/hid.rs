// SPDX-License-Identifier: GPL-3.0-or-later

//! HID access behind traits, so device code can be tested without hardware.

use std::fmt;

/// One HID top-level collection as reported by the operating system.
///
/// A single physical device usually exposes several of these: one per USB
/// interface and, on Windows, one per top-level collection in that interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HidInterface {
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    /// USB interface number, or -1 when the platform does not report one.
    pub interface_number: i32,
    pub usage_page: u16,
    pub usage: u16,
    /// Platform-specific path used to open this collection.
    pub path: String,
}

#[derive(Debug)]
pub enum HidError {
    /// The underlying HID library reported a failure.
    Backend(String),
}

impl fmt::Display for HidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HidError::Backend(msg) => write!(f, "HID backend error: {msg}"),
        }
    }
}

impl std::error::Error for HidError {}

/// Lists the HID collections currently present. Read-only.
pub trait HidEnumerator {
    fn enumerate(&self) -> Result<Vec<HidInterface>, HidError>;
}

/// [`HidEnumerator`] backed by the `hidapi` crate.
pub struct HidApiEnumerator {
    api: hidapi::HidApi,
}

impl HidApiEnumerator {
    pub fn new() -> Result<Self, HidError> {
        let api = hidapi::HidApi::new().map_err(|e| HidError::Backend(e.to_string()))?;
        Ok(Self { api })
    }
}

impl HidEnumerator for HidApiEnumerator {
    fn enumerate(&self) -> Result<Vec<HidInterface>, HidError> {
        Ok(self
            .api
            .device_list()
            .map(|d| HidInterface {
                vendor_id: d.vendor_id(),
                product_id: d.product_id(),
                manufacturer: non_empty(d.manufacturer_string()),
                product: non_empty(d.product_string()),
                interface_number: d.interface_number(),
                usage_page: d.usage_page(),
                usage: d.usage(),
                path: d.path().to_string_lossy().into_owned(),
            })
            .collect())
    }
}

fn non_empty(s: Option<&str>) -> Option<String> {
    s.map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}
