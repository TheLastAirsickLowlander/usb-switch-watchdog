//! Privacy-safe export (spec §10). Build the public incident from
//! allowlisted structured fields only — never regex raw logs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicIncident {
    pub id: String,
    pub kernel_release: String,
    pub distro: String,
    pub controller_vendor: String,
    pub controller_device: String,
    /// Labels like `controller-A`, `device-A`; no PCI addresses / serials.
    pub labels: HashMap<String, String>,
    pub event_counts: HashMap<String, u32>,
    pub safety_decision: String,
    pub outcome: String,
}

pub fn label_for(index: usize, kind: &str) -> String {
    format!("{kind}-{}", (b'A' + index as u8) as char)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_contains_no_raw_identifiers() {
        let inc = PublicIncident {
            id: "inc-test".into(),
            kernel_release: "6.9.1".into(),
            distro: "omarchy".into(),
            controller_vendor: "8086".into(),
            controller_device: "7ae0".into(),
            labels: HashMap::from([
                ("controller".into(), label_for(0, "controller")),
                ("device".into(), label_for(0, "device")),
            ]),
            event_counts: HashMap::from([("cmd-timeout".into(), 2)]),
            safety_decision: "allow".into(),
            outcome: "recovered-provisionally".into(),
        };
        let json = serde_json::to_string(&inc).unwrap();
        for banned in ["05e3", "7ae0-serial", "home/", "192.168.", "machine-id"] {
            assert!(!json.contains(banned), "leaked {banned}");
        }
    }
}
