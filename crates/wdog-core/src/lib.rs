//! Shared types for the USB switch watchdog (spec §3–§10).
//!
//! Deep module: small interface (`Config`, `Detector`, `Safety`, `Export`),
//! all parsing / attribution / redaction logic hidden inside.

pub mod config;
pub mod detection;
pub mod export;
pub mod safety;
pub mod status;

pub use config::{Config, Enrollment, RecoveryMode};
pub use detection::{Detector, DetectorEvent, IncidentCandidate};
pub use export::PublicIncident;
pub use safety::{SafetyDecision, SafetyReport};
pub use status::StatusDoc;
