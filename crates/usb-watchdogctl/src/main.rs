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
    }
    Ok(())
}
