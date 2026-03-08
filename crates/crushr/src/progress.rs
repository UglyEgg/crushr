//! Progress reporting primitives for library and CLI.
//!
//! The library emits progress events through a `ProgressSink`. The CLI implements sinks
//! using rich UI (progress bars, TUI, etc). Embedders can provide their own sink or use `NullProgressSink`.

use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ProgressOp {
    Pack,
    Append,
    Extract,
    Recover,
    Salvage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ProgressPhase {
    ScanInputs,
    TrainDict,
    Compress,
    BuildIndex,
    WriteTail,
    Decompress,
    WriteFiles,
    Other,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ProgressEvent {
    /// Begin an operation. `total_bytes` is best-effort and may be zero if unknown.
    Start {
        op: ProgressOp,
        phase: ProgressPhase,
        total_bytes: u64,
    },
    /// Advance processed bytes.
    AdvanceBytes { bytes: u64 },
    /// Change phase (optionally update total bytes).
    Phase {
        phase: ProgressPhase,
        total_bytes: Option<u64>,
    },
    /// Set a status message.
    Message { msg: String },
    /// Finish operation.
    Finish { ok: bool },
}

pub trait ProgressSink: Send + Sync {
    fn on_event(&self, ev: ProgressEvent);
}

#[derive(Default)]
#[allow(dead_code)]
pub struct NullProgressSink;

impl ProgressSink for NullProgressSink {
    fn on_event(&self, _ev: ProgressEvent) {}
}

pub type SharedSink = Arc<dyn ProgressSink>;
