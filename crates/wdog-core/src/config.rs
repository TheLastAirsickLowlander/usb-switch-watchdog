use serde::{Deserialize, Serialize};

/// Recovery policy. Default is observe-only (spec §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RecoveryMode {
    #[default]
    Observe,
    Notify,
    Automatic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enrollment {
    /// Enrolled PCI address, e.g. `0000:00:14.0`.
    pub pci_address: String,
    pub vendor_id: String,
    pub device_id: String,
    /// Opaque hardware identity hash; mismatch disables automation (§5).
    pub hw_identity: String,
    pub driver: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub enrollment: Enrollment,
    #[serde(default)]
    pub mode: RecoveryMode,
    #[serde(default = "d60")]
    pub detection_window_secs: u64,
    #[serde(default = "d2")]
    pub min_cmd_timeouts: u32,
    #[serde(default = "d15")]
    pub evidence_freshness_secs: u64,
    #[serde(default = "d30")]
    pub verify_window_secs: u64,
    #[serde(default = "d30m")]
    pub cooldown_secs: u64,
    #[serde(default = "d2max")]
    pub max_attempts_per_24h: u32,
    pub omarchy_widget: bool,
}

fn d60() -> u64 { 60 }
fn d2() -> u32 { 2 }
fn d15() -> u64 { 15 }
fn d30() -> u64 { 30 }
fn d30m() -> u64 { 1800 }
fn d2max() -> u32 { 2 }

impl Default for Config {
    fn default() -> Self {
        Self {
            enrollment: Enrollment {
                pci_address: "0000:00:14.0".into(),
                vendor_id: "8086".into(),
                device_id: "7ae0".into(),
                hw_identity: String::new(),
                driver: "xhci_hcd".into(),
            },
            mode: RecoveryMode::Observe,
            detection_window_secs: d60(),
            min_cmd_timeouts: d2(),
            evidence_freshness_secs: d15(),
            verify_window_secs: d30(),
            cooldown_secs: d30m(),
            max_attempts_per_24h: d2max(),
            omarchy_widget: true,
        }
    }
}
