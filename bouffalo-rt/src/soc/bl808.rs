//! BL808 tri-core heterogeneous Wi-Fi 802.11b/g/n, Bluetooth 5, Zigbee AIoT system-on-chip.

mod entry;
mod firmware_header;
mod peripherals;
mod trap;

pub use firmware_header::*;
pub use peripherals::*;
pub use trap::*;
