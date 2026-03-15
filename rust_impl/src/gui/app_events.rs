use std::ffi::os_str::Display;
// app_event.rs - Event enum and EventBus for MainTraclusDL to communicate with GUI and Logger
use std::fmt;
use std::sync::mpsc::{self, Receiver, Sender};

// ─────────────────────────────────────────────
// AppEvent enum : events emitted by MainTraclusDL to report progress and results
// ─────────────────────────────────────────────
#[derive(Debug, Clone)]
pub enum AppEvent {
    LoadComplete {
        desire_line_count: usize,
        correlation_percent: f64,
    },

    ComputationStart {
        computation_type: ComputationType,
        max_progress: usize,
        additional_info: Option<String>,
    },

    ComputationProgress {
        computation_type: ComputationType,
        increment_progress: usize,
    },

    ComputationComplete {
        computation_type: ComputationType,
    },

    /// Emitted on any unrecoverable error inside a task
    Error(AppError),
}

// ─────────────────────────────────────────────
// AppError enum : error types emitted by MainTraclusDL
// ─────────────────────────────────────────────
#[derive(Debug, Clone)]
pub enum AppError {
    NoRawStorage,
    NoClustStorage,
    IoError(String), // variants can still carry dynamic data when needed
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            AppError::NoRawStorage => "No desire line loaded. Please load a file first.",
            AppError::NoClustStorage => {
                "No clustered trajectories available. Please run clustering first."
            }
            AppError::IoError(msg) => msg,
        };
        write!(f, "{}", msg)
    }
}

// ─────────────────────────────────────────────
// ComputationType enum : types of computations that can be performed
// ─────────────────────────────────────────────
#[derive(Debug, Clone)]
pub enum ComputationType {
    InputLoading,
    Clustering,
    RemoveDuplicates,
}

impl fmt::Display for ComputationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            ComputationType::InputLoading => "Loading Input",
            ComputationType::Clustering => "Clustering",
            ComputationType::RemoveDuplicates => "Removing Duplicates",
        };
        write!(f, "{}", msg)
    }
}
// ─────────────────────────────────────────────
// Event : a simple fan-out broadcast channel for AppEvents
// ─────────────────────────────────────────────

pub struct ComputationEvent {
    subscribers: Vec<Sender<AppEvent>>,
}

impl ComputationEvent {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self) -> Receiver<AppEvent> {
        let (tx, rx) = mpsc::channel();
        self.subscribers.push(tx);
        rx
    }

    pub fn emit(&mut self, event: AppEvent) {
        // retain keeps only the senders whose send() succeeded
        self.subscribers.retain(|tx| tx.send(event.clone()).is_ok());
    }

    pub fn has_subscribers(&self) -> bool {
        !self.subscribers.is_empty()
    }
}
