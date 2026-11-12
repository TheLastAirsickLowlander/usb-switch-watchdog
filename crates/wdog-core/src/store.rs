//! On-disk incident history (spec §10).
//!
//! The daemon appends one JSON object per line; readers skip corrupt
//! lines. Missing file = no incidents yet, never an error.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub fn dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(xdg).join("usb-switch-watchdog")
    } else {
        PathBuf::from(env!("HOME")).join(".local/share/usb-switch-watchdog")
    }
}

pub fn incidents_path() -> PathBuf {
    dir().join("incidents.jsonl")
}

/// One safety-gate evaluation, recorded before acting on it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEntry {
    pub at_unix: u64,
    pub gate: String,
    pub verdict: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// One thing the watchdog did, in order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionEntry {
    pub at_unix: u64,
    pub kind: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentRecord {
    pub id: String,
    pub started_unix: u64,
    pub mode: String,
    pub timeouts: u32,
    pub failures: u32,
    #[serde(default)]
    pub decisions: Vec<DecisionEntry>,
    #[serde(default)]
    pub actions: Vec<ActionEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
}

pub fn parse_incidents(text: &str) -> Vec<IncidentRecord> {
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

pub fn load_incidents() -> Vec<IncidentRecord> {
    let Ok(text) = std::fs::read_to_string(incidents_path()) else {
        return Vec::new();
    };
    parse_incidents(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_corrupt_lines_keeps_good_ones() {
        let rec = IncidentRecord {
            id: "inc-1".into(),
            started_unix: 1,
            mode: "observe".into(),
            timeouts: 2,
            failures: 1,
            decisions: vec![DecisionEntry {
                at_unix: 2,
                gate: "block-device-below".into(),
                verdict: "block".into(),
                reason: Some("sdb attached".into()),
            }],
            actions: vec![ActionEntry {
                at_unix: 3,
                kind: "notify".into(),
                detail: "reset blocked".into(),
            }],
            outcome: Some("blocked".into()),
        };
        let good = serde_json::to_string(&rec).unwrap();
        let text = format!("{good}\n{{nope\n\n{good}\n");
        let v = parse_incidents(&text);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].decisions.len(), 1);
        assert_eq!(v[1].id, "inc-1");
    }

    #[test]
    fn empty_input_is_empty_history() {
        assert!(parse_incidents("").is_empty());
        assert!(parse_incidents("\n  \n").is_empty());
    }
}
