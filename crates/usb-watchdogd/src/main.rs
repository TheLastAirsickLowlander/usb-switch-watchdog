//! `usb-watchdogd`: unprivileged detector (spec §4.1).
//! Phase 1: observe mode — journal cursor + `Detector` + incident store.
//! Recovery requests go to the privileged helper over local IPC only.

use wdog_core::{Config, Detector};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = Config::default();
    let _detector =
        Detector::new(Duration::from_secs(cfg.detection_window_secs), cfg.min_cmd_timeouts);
    println!("usb-watchdogd: observe mode (mode={:?})", cfg.mode);
    println!("enrolled: {} ({}:{})", cfg.enrollment.pci_address, cfg.enrollment.vendor_id, cfg.enrollment.device_id);
    // TODO(phase-1): journal cursor, topology attribution, IPC request.
    Ok(())
}
