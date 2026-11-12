//! Incident state machine (spec §6).
//!
//! Interface: push timestamped, controller-attributed kernel lines,
//! get at most one `IncidentCandidate` per coalesced burst.
//! Bus renumbering is handled by re-resolving attribution on topology
//! change; unattributable lines are ignored.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// A kernel journal line already attributed to the enrolled controller.
/// Attribution happens upstream via current sysfs topology (§6).
#[derive(Debug, Clone)]
pub struct DetectorEvent {
    pub at: Instant,
    pub line: String,
    pub kind: EventKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// `Timeout while waiting for setup device command` on enrolled ctrl.
    CmdTimeout,
    /// Config/enumeration failure with `-110`, addr setup `-62`,
    /// or invalid-context warning.
    DeviceSetupFailure,
    /// Anything else attributable but not trigger-relevant.
    Other,
}

impl EventKind {
    pub fn classify(line: &str) -> Self {
        if line.contains("Timeout while waiting for setup device command") {
            Self::CmdTimeout
        } else if line.contains("can't read configurations, error -110")
            || line.contains("can't add hid device: -110")
            || line.contains("can't set config")
            || line.contains("error -62")
            || line.contains("invalid context state for evaluate command")
        {
            Self::DeviceSetupFailure
        } else {
            Self::Other
        }
    }
}

#[derive(Debug, Clone)]
pub struct IncidentCandidate {
    pub id: String,
    pub timeouts: u32,
    pub failures: u32,
}

pub struct Detector {
    window: Duration,
    min_timeouts: u32,
    events: VecDeque<DetectorEvent>,
    in_incident: bool,
    last_emit: Option<Instant>,
}

impl Detector {
    pub fn new(detection_window: Duration, min_timeouts: u32) -> Self {
        Self {
            window: detection_window,
            min_timeouts,
            events: VecDeque::new(),
            in_incident: false,
            last_emit: None,
        }
    }

    /// Push one attributed event. Returns a candidate once per burst.
    /// Recovery's own messages must be filtered by the caller (§6).
    pub fn push(&mut self, ev: DetectorEvent) -> Option<IncidentCandidate> {
        let now = ev.at;
        self.events.push_back(ev);
        while let Some(front) = self.events.front() {
            if now.duration_since(front.at) <= self.window {
                break;
            }
            self.events.pop_front();
        }
        // Coalesce: one candidate until the window drains with no trigger.
        let timeouts = self
            .events
            .iter()
            .filter(|e| e.kind == EventKind::CmdTimeout)
            .count() as u32;
        let failures = self
            .events
            .iter()
            .filter(|e| e.kind == EventKind::DeviceSetupFailure)
            .count() as u32;
        if timeouts >= self.min_timeouts.max(2) && failures >= 1 {
            if self.in_incident {
                return None;
            }
            // Never re-trigger on replayed history: one emit per window.
            if let Some(last) = self.last_emit {
                if now.duration_since(last) < self.window {
                    // Still inside the previous burst; coalesce.
                    self.in_incident = true;
                    return None;
                }
            }
            self.in_incident = true;
            self.last_emit = Some(now);
            return Some(IncidentCandidate {
                id: format!("inc-{:x}", now.elapsed().as_nanos() & 0xffff),
                timeouts,
                failures,
            });
        }
        if timeouts == 0 {
            self.in_incident = false;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(kind: EventKind, at: Instant) -> DetectorEvent {
        DetectorEvent { at, line: String::new(), kind }
    }

    #[test]
    fn normal_switching_never_triggers() {
        let t = Instant::now();
        let mut d = Detector::new(Duration::from_secs(60), 2);
        for i in 0..10 {
            assert!(d.push(ev(EventKind::Other, t + Duration::from_secs(i))).is_none());
        }
    }

    #[test]
    fn isolated_single_timeout_never_triggers() {
        let t = Instant::now();
        let mut d = Detector::new(Duration::from_secs(60), 2);
        assert!(d.push(ev(EventKind::CmdTimeout, t)).is_none());
        assert!(d.push(ev(EventKind::DeviceSetupFailure, t)).is_none());
    }

    #[test]
    fn observed_sequence_triggers_once() {
        let t = Instant::now();
        let mut d = Detector::new(Duration::from_secs(60), 2);
        assert!(d.push(ev(EventKind::CmdTimeout, t)).is_none());
        assert!(d.push(ev(EventKind::CmdTimeout, t + Duration::from_secs(5))).is_none());
        let c = d.push(ev(EventKind::DeviceSetupFailure, t + Duration::from_secs(6)));
        assert!(c.is_some());
        // Replay / duplicate within window coalesces.
        assert!(d.push(ev(EventKind::CmdTimeout, t + Duration::from_secs(7))).is_none());
    }
}
