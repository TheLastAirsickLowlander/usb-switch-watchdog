//! Machine-readable status for `usb-watchdogctl status --json` (spec §4.3).
//!
//! The Omarchy bar widget polls this. Allowlisted fields only — the same
//! privacy rule as incident export (§10): no serials, hostnames, IPs.

use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusDoc {
    pub schema: u32,
    pub mode: String,
    pub paused: bool,
    pub enrolled_pci: String,
    pub driver: String,
    /// `unknown` until the daemon reports; framework phase has no daemon yet.
    pub automation: String,
    pub incidents_24h: u32,
    pub last_outcome: Option<String>,
}

impl StatusDoc {
    pub fn from_config(cfg: &Config) -> Self {
        Self {
            schema: 1,
            mode: format!("{:?}", cfg.mode).to_lowercase(),
            paused: false,
            enrolled_pci: cfg.enrollment.pci_address.clone(),
            driver: cfg.enrollment.driver.clone(),
            automation: "disabled-observe".into(),
            incidents_24h: 0,
            last_outcome: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_doc_schema_is_stable_and_clean() {
        let doc = StatusDoc::from_config(&Config::default());
        let v = serde_json::to_value(&doc).unwrap();
        let mut keys: Vec<_> = v.as_object().unwrap().keys().cloned().collect();
        keys.sort();
        assert_eq!(
            keys,
            ["automation", "driver", "enrolled_pci", "incidents_24h", "last_outcome",
                "mode", "paused", "schema"]
        );
        let raw = serde_json::to_string(&doc).unwrap();
        for banned in ["serial", "home/", "machine-id", "boot-id", "192.168."] {
            assert!(!raw.to_lowercase().contains(banned), "leaked {banned}");
        }
    }
}
