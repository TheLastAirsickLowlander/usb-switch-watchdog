# Omarchy bar plugin (`wdog.usb-switch`)

Native Quickshell bar widget in first-party style: one Nerd Font USB
glyph (`󰕓`) in the host's own colors. State lives in the tooltip and
in the button's `active` highlight. Click opens `wdog-panel`, the Rune
desktop app with incident history, decisions, and actions.

Read-only: polls `usb-watchdogctl status --json` every 15 s. Every
failure path (missing binary, bad JSON, daemon down) clears the
highlight — a broken widget must never break the bar.

## Install

```bash
cp -r omarchy-plugin ~/.config/omarchy/plugins/wdog.usb-switch
# ensure usb-watchdogctl + wdog-panel are on PATH (cargo install)
omarchy plugin enable wdog.usb-switch --section right
```

To move it later:

```bash
omarchy bar move wdog.usb-switch --section right
```

Note: the shell caches widget components by URL, so editing the QML
in place does not reload it — run `omarchy-restart-shell` after
changing plugin files.

## States

| Glyph | Meaning |
|---|---|
| plain | monitoring (observe/automatic) or unknown |
| highlighted | needs attention: paused, failed/stalled, or reset in progress |
| tooltip | mode, automation, last outcome, present-tense activity |

Anything the widget doesn't understand renders plain. A broken widget
must never break the bar.
