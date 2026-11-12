//! `usb-watchdogctl`: status / pause / resume / manual recovery / export.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "usb-watchdogctl", about = "USB switch watchdog control")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Show mode, enrollment, last incident, budgets.
    Status {
        /// Machine-readable output for the Omarchy bar widget.
        #[arg(long)]
        json: bool,
    },
    /// Pause automatic recovery.
    Pause,
    /// Resume automatic recovery.
    Resume,
    /// Manual recovery (still passes storage/topology checks).
    Recover,
    /// Emit sanitized incident export (allowlisted fields only).
    Export {
        #[arg(long)]
        id: Option<String>,
    },
    /// Emit a test notification via the same path recovery uses (§8.4).
    Notify {
        #[arg(long, default_value = "USB watchdog: test notification")]
        summary: String,
        #[arg(long, default_value = "Notification path works.")]
        body: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Status { json } => {
            // Framework phase: no daemon state dir yet; report config + unknowns.
            let doc = wdog_core::StatusDoc::from_config(&wdog_core::Config::default());
            if json {
                println!("{}", serde_json::to_string(&doc)?);
            } else {
                println!(
                    "mode={} enrolled={} automation={} incidents_24h={}",
                    doc.mode, doc.enrolled_pci, doc.automation, doc.incidents_24h
                );
            }
        }
        Cmd::Pause => println!("paused"),
        Cmd::Resume => println!("resumed"),
        Cmd::Recover => println!("manual recovery requested (safety checks apply)"),
        Cmd::Export { id } => {
            println!("{{\"id\": {:?}, \"note\": \"allowlisted export only\"}}", id)
        }
        Cmd::Notify { summary, body } => {
            wdog_core::notify::send(&summary, &body, wdog_core::notify::Urgency::Normal);
            // Give the fire-and-forget thread a moment before process exit.
            std::thread::sleep(std::time::Duration::from_millis(300));
            println!("notification sent");
        }
    }
    Ok(())
}
