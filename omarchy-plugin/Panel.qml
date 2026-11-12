import QtQuick
import Quickshell
import Quickshell.Io
import qs.Commons
import qs.Ui

// Bar widget for the USB switch watchdog, in first-party style: one
// Nerd Font glyph in the host's own colors. State lives in the tooltip
// and in the `active` highlight, never in custom colors or text labels.
// Read-only: polls `usb-watchdogctl status --json`. Every failure path
// (missing binary, bad JSON, daemon down) clears the highlight — a broken
// widget must never break the bar.
// Root is Ui.Panel like other third-party bar widgets (omdrop); BarWidget
// is first-party-only and rejects custom properties from plugins.
Panel {
  id: root
  moduleName: "wdog.usb-switch"
  // Overridable via the plugin schema.
  property int pollSeconds: 15

  // unknown | observe | auto | active | paused | incident
  property string wstate: "unknown"
  property string detail: "watchdog state unknown"


  implicitWidth: button.implicitWidth
  implicitHeight: button.implicitHeight

  // Lit only when the watchdog wants attention: paused, failed/stalled,
  // or a reset in progress. Steady states stay the plain bar glyph.
  readonly property bool needsAttention:
    wstate === "paused" || wstate === "incident" || wstate === "active"

  function applyJson(text) {
    var doc;
    try {
      doc = JSON.parse(text);
    } catch (e) {
      root.wstate = "unknown";
      root.detail = "unparseable status";
      return;
    }
    if (!doc || doc.schema !== 1 || !doc.mode) {
      root.wstate = "unknown";
      root.detail = "unsupported status schema";
      return;
    }
    var summary = "mode " + doc.mode + ", automation " + doc.automation;
    if (doc.last_outcome)
      summary += ", last " + doc.last_outcome;
    if (doc.activity === "recovering") {
      root.wstate = "active";
      root.detail = "reset in progress — devices will briefly drop";
    } else if (doc.paused) {
      root.wstate = "paused";
      root.detail = "automatic recovery paused (" + summary + ")";
    } else if (doc.last_outcome === "failed" || doc.last_outcome === "stalled"
        || doc.automation === "disabled-failure") {
      root.wstate = "incident";
      root.detail = "needs review (" + summary + ")";
    } else if (doc.mode === "automatic") {
      root.wstate = "auto";
      root.detail = "monitoring, recovery armed (" + summary + ")";
    } else {
      root.wstate = "observe";
      root.detail = "monitoring only (" + summary + ")";
    }
  }

  Process {
    id: poll
    command: ["usb-watchdogctl", "status", "--json"]
    stdout: StdioCollector {}
    onExited: function (exitCode) {
      if (exitCode !== 0)
        root.applyJson("");
      else
        root.applyJson(poll.stdout.text);
      pollTimer.restart();
    }
  }

  Timer {
    id: pollTimer
    interval: Math.max(5, root.pollSeconds) * 1000
    onTriggered: poll.running = true
  }

  // Click opens the Rune detail panel (richer incident history,
  // pause/resume, export). Fire-and-forget: the panel owns its lifetime.
  Process {
    id: launcher
    command: ["wdog-panel"]
  }

  BarIconButton {
    id: button
    anchors.fill: parent
    bar: root.bar
    // Material Design "usb" glyph, same icon font as the other bar icons.
    text: "󰕓"
    active: root.needsAttention
    tooltipText: "USB watchdog: " + root.detail
    onPressed: function (buttonCode) {
      if (buttonCode === Qt.LeftButton)
        launcher.running = true;
    }
  }

  Component.onCompleted: poll.running = true
}
