// app_event.rs - Event enum and EventBus for MainTraclusDL to communicate with GUI and Logger
use eframe::egui::{self, Color32};
use std::{fmt, time::Instant};

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
    },

    ComputationProgress {
        computation_type: ComputationType,
        increment_progress: usize,
    },

    ComputationComplete {
        computation_type: ComputationType,
    },

    PrintInfo {
        messages: Vec<String>,
    },

    PerfTimer{
        event_label: &'static str,
        exact_instant: Instant,
        is_start: bool,
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
                "No clustered trajectories available. Please run computation first."
            }
            AppError::IoError(msg) => msg,
        };
        write!(f, "{}", msg)
    }
}

// ─────────────────────────────────────────────
// ComputationType enum : types of computations that can be performed
// ─────────────────────────────────────────────
#[derive(Clone, PartialEq, Eq)]
pub enum ComputationType {
    Clustering = 1,
    RemoveDuplicates = 2,
    CreateOutputs = 3,
    NotComputing = 0, // default value for ViewModel when no computation is running
}

impl fmt::Debug for ComputationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            ComputationType::Clustering => "Clustering",
            ComputationType::RemoveDuplicates => "Removing Duplicates",
            ComputationType::CreateOutputs => "Creating Output Files",
            ComputationType::NotComputing => "Not Computing",
        };
        write!(f, "{}", msg)
    }
}

impl ComputationType {
    pub fn color(&self) -> egui::Color32 {
        match self {
            ComputationType::Clustering => Color32::from_rgb(39, 115, 38), // green
            ComputationType::RemoveDuplicates => Color32::from_rgb(196, 148, 81), // light orange
            ComputationType::CreateOutputs => Color32::from_rgb(0, 0, 255), // blue
            ComputationType::NotComputing => Color32::from_rgb(128, 128, 128), // gray
        }
    }
}
