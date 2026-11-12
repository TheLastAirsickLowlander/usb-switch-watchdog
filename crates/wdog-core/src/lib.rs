//! Shared types for the USB switch watchdog (spec §3–§10).
//!
//! Deep module: small interface (`Config`, `Detector`, `Safety`, `Export`),
//! all parsing / attribution / redaction logic hidden inside.

pub mod config;
pub mod detection;
pub mod export;
pub mod safety;

pub use config::{Config, Enrollment, RecoveryMode};
pub use detection::{Detector, DetectorEvent, IncidentCandidate};
pub use safety::{SafetyDecision, SafetyReport};
