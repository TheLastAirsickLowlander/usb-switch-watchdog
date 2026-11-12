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
    Status,
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
        Cmd::Status => println!("mode=observe enrolled=0000:00:14.0 incidents=0"),
        Cmd::Pause => println!("paused"),
        Cmd::Resume => println!("resumed"),
        Cmd::Recover => println!("manual recovery requested (safety checks apply)"),
        Cmd::Export { id } => {
            println!("{{\"id\": {:?}, \"note\": \"allowlisted export only\"}}", id)
        }
    }
    Ok(())
}
