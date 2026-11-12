# Omarchy bar plugin (`wdog.usb-switch`)

Native Quickshell bar widget. Read-only: polls `usb-watchdogctl status
--json` every 15 s and renders one state dot. Click opens `wdog-panel`.

## Install

```bash
cp -r omarchy-plugin ~/.config/omarchy/plugins/wdog.usb-switch
# ensure usb-watchdogctl + wdog-panel are on PATH (cargo install)
```

Add the widget to the bar in `~/.config/omarchy/shell.json`, e.g. in
`tray` or `right`:

```json
{ "id": "wdog.usb-switch" }
```

Then reload the shell (re-login, or whatever your build exposes —
`omarchy bar --help` may offer move/add helpers). To move it later:

```bash
omarchy bar move wdog.usb-switch --section right
```

## States

| Dot | Meaning |
|---|---|
| grey | unknown — ctl missing, daemon down, or bad output |
| blue | observe — monitoring only |
| green | automatic — recovery armed |
| amber | paused |
| red | last recovery failed/stalled, needs review |

Anything the widget doesn't understand renders grey. A broken widget
must never break the bar.
