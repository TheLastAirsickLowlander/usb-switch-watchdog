import QtQuick
import Quickshell
import Quickshell.Io
import qs.Commons as Commons

// Bar widget for the USB switch watchdog. Read-only: it polls
// `usb-watchdogctl status --json` and renders one state dot. Every
// failure path (missing binary, bad JSON, daemon down) lands on
// "unknown" — a broken widget must never break the bar.
Item {
  id: root

  // Configurable via the plugin schema.
  property int pollSeconds: 15

  // unknown | observe | auto | paused | incident
  property string wstate: "unknown"
  property string detail: "watchdog state unknown"

  implicitWidth: row.width
  implicitHeight: row.height

  Row {
    id: row
    spacing: 6

    Rectangle {
      id: dot
      anchors.verticalCenter: parent.verticalCenter
      width: 9
      height: 9
      radius: 4.5
      color: {
        switch (root.wstate) {
        case "observe": return "#7aa2f7";
        case "auto": return "#9ece6a";
        case "paused": return "#e0af68";
        case "incident": return "#f7768e";
        default: return "#888888";
        }
      }
    }

    Text {
      anchors.verticalCenter: parent.verticalCenter
      text: "USB"
      color: Commons.Color.foreground
    }
  }

  function applyJson(text) {
    var doc;
    try {
      doc = JSON.parse(text);
    } catch (e) {
      root.wstate = "unknown";
      root.detail = "unparseable status";
      return;
    }
    if (doc.schema !== 1 || !doc.mode) {
      root.wstate = "unknown";
      root.detail = "unsupported status schema";
      return;
    }
    if (doc.paused) {
      root.wstate = "paused";
      root.detail = "automatic recovery paused";
    } else if (doc.last_outcome === "failed" || doc.last_outcome === "stalled"
        || doc.automation === "disabled-failure") {
      root.wstate = "incident";
      root.detail = "last recovery " + doc.last_outcome;
    } else if (doc.mode === "automatic") {
      root.wstate = "auto";
      root.detail = "monitoring, automatic recovery armed";
    } else {
      root.wstate = "observe";
      root.detail = "monitoring only, no automatic reset";
    }
  }

  Process {
    id: poll
    command: ["usb-watchdogctl", "status", "--json"]
    stdout: StdioCollector {}
    onExited: {
      if (exitCode !== 0)
        root.applyJson("");
      else
        root.applyJson(stdout.text);
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

  MouseArea {
    anchors.fill: parent
    acceptedButtons: Qt.LeftButton
    accessible.name: "USB watchdog: " + root.detail
    onClicked: launcher.running = true
  }

  Component.onCompleted: poll.running = true
}
