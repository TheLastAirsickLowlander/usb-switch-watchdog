// UsbWatchdog.qml — optional native Quickshell bar widget.
// Polls `usb-watchdogctl status --json` (1-shot, no daemon dependency).
// The Rune `wdog-panel` app is the richer companion; use one or both.
import QtQuick
import Quickshell
import Quickshell.Io

Text {
    id: root
    text: "● USB"
    Process {
        id: poll
        command: ["usb-watchdogctl", "status"]
        running: true
        stdout: StdioCollector {}
        onExited: pollTimer.restart()
    }
    Timer {
        id: pollTimer
        interval: 15000
        onTriggered: poll.running = true
    }
}
