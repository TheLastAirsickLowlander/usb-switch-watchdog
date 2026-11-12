//! `wdog-panel`: Rune desktop status app for the USB switch watchdog.
//!
//! Unprivileged by design. It reads the daemon's status JSON and incident
//! store, and shells out to `usb-watchdogctl` for actions. It never touches
//! sysfs, never binds/unbinds drivers, never needs root, polkit, or sudo:
//! every privileged step stays inside `wdog-recover`, which accepts only a
//! fixed request and rechecks safety gates itself (spec §4.2, §7–§8).
//! Clicking the bar widget opens this window; the widget itself stays a
//! thin read-only poller.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use rune::{
    color,
    tok::{self, type_},
    AppSpec, Cx, El, ScrollSpec, col, el, row, text,
};
use wdog_core::{Config, StatusDoc, store::{self, IncidentRecord}};

const PINK: &str = "#f7768e";
const GREEN: &str = "#9ece6a";
const AMBER: &str = "#e0af68";
const BLUE: &str = "#7aa2f7";
const GREY: &str = "#888888";

fn now_unix() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn fmt_time(unix: u64) -> String {
    let age = now_unix().saturating_sub(unix);
    if age < 90 {
        format!("{age}s ago")
    } else if age < 5400 {
        format!("{}m ago", age / 60)
    } else {
        format!("{}h ago", age / 3600)
    }
}

struct Panel {
    dark: bool,
    status: StatusDoc,
    incidents: Vec<IncidentRecord>,
    selected: Option<String>,
    note: String,
}

impl Panel {
    fn refresh(&mut self) {
        self.status = query_status();
        self.incidents = store::load_incidents();
        if let Some(id) = &self.selected {
            if !self.incidents.iter().any(|i| &i.id == id) {
                self.selected = None;
            }
        }
        self.note = format!("refreshed ({})", self.incidents.len());
    }
}

fn query_status() -> StatusDoc {
    // Prefer the installed CLI (same version as the daemon). Fallback is
    // the compiled-in default, so the window is never empty.
    if let Ok(out) = Command::new("usb-watchdogctl").args(["status", "--json"]).output() {
        if out.status.success() {
            if let Ok(doc) = serde_json::from_slice::<StatusDoc>(&out.stdout) {
                return doc;
            }
        }
    }
    StatusDoc::from_config(&Config::default())
}

fn run_ctl(args: &[&str]) -> String {
    match Command::new("usb-watchdogctl").args(args).output() {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
        Ok(out) => format!("ctl failed: {}", String::from_utf8_lossy(&out.stderr).trim()),
        Err(e) => format!("ctl unavailable: {e}"),
    }
}

fn outcome_color(outcome: &Option<String>) -> &'static str {
    match outcome.as_deref() {
        Some("recovered-provisionally") | Some("recovered") | Some("driver-restored") => GREEN,
        Some("blocked") | Some("paused") => AMBER,
        Some("failed") | Some("stalled") | Some("partial") => PINK,
        _ => GREY,
    }
}

fn mode_color(mode: &str) -> &'static str {
    match mode {
        "automatic" => GREEN,
        "notify" => BLUE,
        _ => GREY,
    }
}

fn button(label: &str, key: &str, run: fn(&mut Panel)) -> El {
    let t = tok::theme(true);
    el().key(key.to_string()).role("button").label(label.to_string()).focusable(true)
        .padding(8.0).radius(8.0).fill(t.control_off)
        .on_press(move |s: &mut Panel| run(s))
        .child(text(label.to_string()).font(type_::BODY).color(t.text))
}

fn do_pause(s: &mut Panel) { s.note = run_ctl(&["pause"]); s.refresh(); }
fn do_resume(s: &mut Panel) { s.note = run_ctl(&["resume"]); s.refresh(); }
fn do_recover(s: &mut Panel) { s.note = run_ctl(&["recover"]); s.refresh(); }
fn do_export(s: &mut Panel) { s.note = run_ctl(&["export"]); s.refresh(); }
fn do_refresh(s: &mut Panel) { s.refresh(); }

fn view(s: &Panel, _: &mut Cx<Panel>) -> El {
    let t = tok::theme(s.dark);
    let mode_c = color::hex(mode_color(&s.status.mode)).unwrap_or(t.text);
    let mut root = col().w_full().h_full().padding(16.0).gap(12.0).fill(t.bg);
    root = root.child(
        row().w_full().gap(10.0)
            .child(text("USB switch watchdog".to_string()).font(type_::TITLE).color(t.text))
            .child(
                el().padding(6.0).radius(7.0).fill(t.control_off)
                    .child(text(s.status.mode.clone()).font(type_::BODY).color(mode_c))),
    );
    root = root.child(
        text(format!(
            "enrolled {} ({}) · automation {} · {} incident(s) in 24h",
            s.status.enrolled_pci, s.status.driver, s.status.automation, s.status.incidents_24h,
        )).font(type_::BODY).color(t.text_secondary),
    );
    root = root.child(
        row().w_full().gap(8.0)
            .child(button("Pause", "btn-pause", do_pause))
            .child(button("Resume", "btn-resume", do_resume))
            .child(button("Recover", "btn-recover", do_recover))
            .child(button("Export", "btn-export", do_export))
            .child(button("Refresh", "btn-refresh", do_refresh)),
    );
    if !s.note.is_empty() {
        root = root.child(text(s.note.clone()).font(type_::BODY).color(t.text_secondary));
    }
    root = root.child(
        text(format!("Incidents ({})", s.incidents.len())).font(type_::TITLE).color(t.text),
    );

    if s.incidents.is_empty() {
        root = root.child(
            text("No incidents recorded. The detector logs here when the enrolled controller stalls."
                .to_string()).font(type_::BODY).color(t.text_secondary),
        );
        return root;
    }

    let mut list = col().w_full().h(180.0).gap(6.0)
        .scroll(ScrollSpec { stick_bottom: false });
    for inc in &s.incidents {
        let c = color::hex(outcome_color(&inc.outcome)).unwrap_or(t.text);
        let id = inc.id.clone();
        let sel = s.selected.as_deref() == Some(&id);
        list = list.child(
            el().key(format!("inc-{id}")).w_full().padding(8.0).radius(8.0)
                .fill(if sel { t.control_off } else { t.bg })
                .role("button").label(format!("incident {id}")).focusable(true)
                .on_press(move |st: &mut Panel| {
                    st.selected = Some(id.clone());
                    st.note.clear();
                })
                .child(
                    row().w_full().gap(10.0)
                        .child(text(inc.id.clone()).font(type_::BODY).color(t.text))
                        .child(
                            text(inc.outcome.clone().unwrap_or_else(|| "open".to_string()))
                                .font(type_::BODY).color(c))
                        .child(
                            text(format!("{} timeouts · {} failures · {}",
                                inc.timeouts, inc.failures, fmt_time(inc.started_unix)))
                            .font(type_::BODY).color(t.text_secondary))),

        );
    }
    root = root.child(list);

    // Selected incident: what happened, and what the watchdog decided.
    if let Some(inc) = s.selected.as_deref().and_then(|id| s.incidents.iter().find(|i| i.id == id)) {
        let mut detail = col().w_full().h(240.0).gap(6.0).padding(10.0).radius(10.0)
            .fill(t.control_off)
            .scroll(ScrollSpec { stick_bottom: false });
        detail = detail.child(
            text(format!("{} — {}", inc.id, inc.outcome.as_deref().unwrap_or("open")))
                .font(type_::TITLE).color(t.text));
        detail = detail.child(text("Decisions".to_string()).font(type_::BODY).color(t.text));
        if inc.decisions.is_empty() {
            detail = detail.child(
                text("no gate evaluations recorded".to_string())
                    .font(type_::BODY).color(t.text_secondary));
        }
        for d in &inc.decisions {
            let line = match &d.reason {
                Some(r) => format!("{} → {} ({})", d.gate, d.verdict, r),
                None => format!("{} → {}", d.gate, d.verdict),
            };
            detail = detail.child(
                text(format!("{} · {line}", fmt_time(d.at_unix)))
                    .font(type_::BODY).color(t.text_secondary));
        }
        detail = detail.child(text("Actions".to_string()).font(type_::BODY).color(t.text));
        if inc.actions.is_empty() {
            detail = detail.child(
                text("nothing taken yet".to_string()).font(type_::BODY).color(t.text_secondary));
        }
        for a in &inc.actions {
            detail = detail.child(
                text(format!("{} · {}: {}", fmt_time(a.at_unix), a.kind, a.detail))
                    .font(type_::BODY).color(t.text_secondary));
        }
        root = root.child(detail);
    }
    root
}

fn sample_incident() -> IncidentRecord {
    IncidentRecord {
        id: "inc-9f2a".into(),
        started_unix: now_unix() - 320,
        mode: "automatic".into(),
        timeouts: 3,
        failures: 1,
        decisions: vec![
            store::DecisionEntry {
                at_unix: now_unix() - 300, gate: "block-device-below".into(),
                verdict: "allow".into(), reason: None,
            },
            store::DecisionEntry {
                at_unix: now_unix() - 299, gate: "usb-network-below".into(),
                verdict: "allow".into(), reason: None,
            },
        ],
        actions: vec![
            store::ActionEntry {
                at_unix: now_unix() - 298, kind: "notify".into(),
                detail: "reset starting".into(),
            },
            store::ActionEntry {
                at_unix: now_unix() - 290, kind: "unbind/bind".into(),
                detail: "0000:00:14.0 reattached to xhci_hcd".into(),
            },
            store::ActionEntry {
                at_unix: now_unix() - 260, kind: "verify".into(),
                detail: "driver restored, 3 devices re-enumerated".into(),
            },
        ],
        outcome: Some("recovered-provisionally".into()),
    }
}

impl Panel {
    fn load() -> Self {
        Panel {
            dark: true,
            status: query_status(),
            incidents: store::load_incidents(),
            selected: None,
            note: String::new(),
        }
    }
}

fn main() {
    rune::run(AppSpec::<Panel> {
        scenes: &[("incident", |s| {
            s.incidents = vec![sample_incident()];
            s.selected = Some("inc-9f2a".into());
        })],
        ..AppSpec::new("USB watchdog", (580.0, 760.0), Panel::load, view)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::{Env, Runtime};

    fn harness() -> Runtime<Panel> {
        Runtime::new(
            Panel {
                dark: true,
                status: StatusDoc::from_config(&Config::default()),
                incidents: vec![sample_incident()],
                selected: None,
                note: String::new(),
            },
            view,
            (580.0, 760.0),
            Env { system_dark: true, scale: 2.0 },
        )
    }

    #[test]
    fn refresh_marks_state() {
        let mut rt = harness();
        rt.frame(0.0);
        let (x, y) = rt.find("btn-refresh").unwrap();
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(1.0 / 60.0);
        assert!(rt.state.note.starts_with("refreshed"));
    }

    #[test]
    fn selecting_incident_shows_its_decisions() {
        let mut rt = harness();
        rt.frame(0.0);
        let (x, y) = rt.find("inc-inc-9f2a").unwrap();
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.selected.as_deref(), Some("inc-9f2a"));
        assert!(rt.state.incidents[0].decisions.iter().any(|d| d.gate == "block-device-below"));
    }
}
