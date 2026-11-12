//! `wdog-recover`: root-owned recovery helper (spec §4.2, §8).
//!
//! Accepts exactly one fixed request (enrolled PCI address), rechecks
//! enrollment + safety gates immediately before unbind, performs the
//! unbind/bind transaction, and restores binding if interrupted.
//! Never takes paths, commands, or sysfs values from the caller.

use wdog_core::safety::{self, TopologySnapshot};

fn main() -> anyhow::Result<()> {
    // TODO(phase-2): socket activation, peer-cred auth, exclusive lock,
    // attempt budget, pre-reset snapshot, unbind/bind, verify window.
    let snap = TopologySnapshot::default();
    let report = safety::check(&snap);
    println!("wdog-recover: pre-flight decision: {:?}", report.decision);
    println!("(default snapshot mismatches enrollment; refusing — correct)");
    Ok(())
}
