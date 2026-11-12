//! Local user notifications (spec §8.4).
//!
//! Fire-and-forget `notify-send`. Never blocks recovery: the child is
//! spawned onto a throwaway thread that reaps it. Bodies carry state
//! words only — never identifiers (§10).

use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub enum Urgency {
    Low,
    Normal,
    Critical,
}

impl Urgency {
    fn flag(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::Critical => "critical",
        }
    }
}

pub fn send(summary: &str, body: &str, urgency: Urgency) {
    let (summary, body) = (summary.to_owned(), body.to_owned());
    std::thread::spawn(move || {
        let _ = Command::new("notify-send")
            .args([
                "-a",
                "USB watchdog",
                "-u",
                urgency.flag(),
                "-t",
                "8000",
                &summary,
                &body,
            ])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output();
    });
}

/// Pre-reset banner: the controller is about to drop input/audio/etc.
pub fn recovery_starting() {
    send(
        "USB watchdog: resetting controller",
        "Input and USB devices will briefly drop. Do not unplug anything.",
        Urgency::Normal,
    );
}

/// Post-attempt banner. `ok` with the verification outcome word.
pub fn recovery_done(outcome: &str, ok: bool) {
    send(
        "USB watchdog: reset complete",
        &format!("Outcome: {outcome}."),
        if ok { Urgency::Normal } else { Urgency::Critical },
    );
}

/// Blocked attempt: automation refused, with the gate reason.
pub fn recovery_blocked(reason: &str) {
    send(
        "USB watchdog: reset blocked",
        &format!("Safety gate held: {reason}. No hardware touched."),
        Urgency::Critical,
    );
}
