//! `wdog-panel`: Rune top-bar / tray companion (Omarchy/Hyprland).
//!
//! Omarchy's bar is Quickshell QML, not a native embed target, so this is
//! a standalone Rune window + tray pill, NOT a QML plugin:
//!
//! - Idle: small always-on-top pill (layer-shell overlay) showing
//!   `● OK` / `◐ OBSERVED` / `● RECOVERY-BLOCKED`.
//! - Click: opens the status window (mode, enrollment, last incident,
//!   pause/resume/recover buttons shelling out to `usb-watchdogctl`).
//! - The QML fragment in `omarchy-plugin/` mirrors the same state by
//!   polling `usb-watchdogctl status` for users who prefer a native
//!   Quickshell bar widget.
//!
//! Start from Rune's `examples/hello`; verify headless (`--tree` / PNG)
//! per Rune's agent workflow. Pin a `dev` rev in Cargo.toml first.

fn main() -> anyhow::Result<()> {
    println!("wdog-panel: stub — add rune dep (pin dev rev), then implement pill + status window");
    println!("see crates/wdog-panel/src/main.rs docs + omarchy-plugin/UsbWatchdog.qml");
    Ok(())
}
