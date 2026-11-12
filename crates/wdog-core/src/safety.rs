//! Safety gates (spec §7). Pure decision function over a topology
//! snapshot; the privileged helper rechecks immediately before unbind.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TopologySnapshot {
    pub identity_matches: bool,
    pub driver_matches: bool,
    pub has_block_device_below: bool,
    pub has_usb_net_below: bool,
    pub unknown_state: bool,
    pub topology_changed_since_check: bool,
    pub host_suspending_or_shutting_down: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyDecision {
    Allow,
    Block { reason: String },
}

impl SafetyDecision {
    pub fn is_allow(&self) -> bool {
        matches!(self, Self::Allow)
    }
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Allow => None,
            Self::Block { reason } => Some(reason),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyReport {
    pub decision: SafetyDecision,
}

pub fn check(snap: &TopologySnapshot) -> SafetyReport {
    let block = if !snap.identity_matches {
        Some("enrollment-mismatch")
    } else if !snap.driver_matches {
        Some("driver-mismatch")
    } else if snap.host_suspending_or_shutting_down {
        Some("host-transition")
    } else if snap.has_block_device_below {
        Some("block-device-below")
    } else if snap.has_usb_net_below {
        Some("usb-network-below")
    } else if snap.unknown_state {
        Some("unknown-safety-state")
    } else if snap.topology_changed_since_check {
        Some("topology-changed")
    } else {
        None
    };
    SafetyReport {
        decision: match block {
            Some(reason) => SafetyDecision::Block { reason: reason.to_owned() },
            None => SafetyDecision::Allow,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok() -> TopologySnapshot {
        TopologySnapshot {
            identity_matches: true,
            driver_matches: true,
            ..Default::default()
        }
    }

    #[test]
    fn blocks_storage_and_net_and_mismatch() {
        assert!(check(&ok()).decision.is_allow());
        let mut s = ok();
        s.has_block_device_below = true;
        assert_eq!(check(&s).decision.reason(), Some("block-device-below"));
        let mut s = ok();
        s.has_usb_net_below = true;
        assert_eq!(check(&s).decision.reason(), Some("usb-network-below"));
        let mut s = ok();
        s.identity_matches = false;
        assert_eq!(check(&s).decision.reason(), Some("enrollment-mismatch"));
        let mut s = ok();
        s.unknown_state = true;
        assert_eq!(check(&s).decision.reason(), Some("unknown-safety-state"));
    }
}
