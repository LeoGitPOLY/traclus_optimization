// app_event.rs - Event enum and EventBus for MainTraclusDL to communicate with GUI and Logger

use std::sync::mpsc::{self, Receiver, Sender};

// ─────────────────────────────────────────────
// AppEvent enum : events emitted by MainTraclusDL to report progress and results
// ─────────────────────────────────────────────
#[derive(Debug, Clone)]
pub enum AppEvent {
    LoadComplete {
        traj_count: usize,
        correlation_percent: f64,
    },

    ComputationClusteringProgress {
        num_traj_done: usize,
    },

    ComputationComplete {
        total_corridors: usize,
        total_seg: usize,
        total_seg_outside_corridor: usize,
    },

    /// Emitted on any unrecoverable error inside a task
    Error(String),
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
