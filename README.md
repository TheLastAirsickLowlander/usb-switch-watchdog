# usb-switch-watchdog

Linux service that detects the UGREEN-switch USB controller stall
(Intel `8086:7ae0` / `xhci_hcd`) and optionally resets that one
controller without rebooting. Observe-mode first; automatic reset only
after explicit approval + hardware validation.

Spec: see attached design doc in the project brief (observe → helper →
manual → automatic delivery plan).

## Layout

```text
crates/wdog-core      shared types: Config, Detector, Safety, Export
crates/usb-watchdogd  unprivileged detector daemon (observe mode)
crates/wdog-recover   root-owned reset helper (fixed request only)
crates/usb-watchdogctl  CLI: status / pause / resume / recover / export
crates/wdog-panel     Rune GUI pill + status window (top-bar companion)
systemd/              service units
config/               enrollment + policy examples
omarchy-plugin/       optional native Quickshell bar widget (polls ctl)
```

## Bar integration (Omarchy)

Omarchy's bar is Quickshell QML — Rune can't embed in it. Two options:

1. `wdog-panel`: standalone Rune pill (layer-shell overlay) + status
   window. Pin a `dev` rev of `github.com/anomalyco/rune` in
   `crates/wdog-panel/Cargo.toml`, start from Rune `examples/hello`.
2. `omarchy-plugin/UsbWatchdog.qml`: native bar widget polling the CLI.

## Status
Framework skeleton. `wdog-core` detection/safety/export logic + unit tests pass; binaries are stubbed per the delivery plan.
