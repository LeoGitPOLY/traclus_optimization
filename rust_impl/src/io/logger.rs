// logger.rs - Event subscriber that prints AppEvents to stdout
//
// The logger runs on its own dedicated std::thread
// CPU usage is kept low with zero busy-wait — the thread parks completely between events.

use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Instant;

use crate::gui::app_events::AppEvent;

pub struct Logger;

impl Logger {
    /// Spawn the logger thread.
    /// `rx` is the Receiver obtained from EventBus::subscribe().
    pub fn start(rx: Receiver<AppEvent>) {
        thread::Builder::new()
            .name("traclus-logger".to_string())
            .spawn(move || Self::run(rx))
            .expect("failed to spawn logger thread");
    }

    fn run(rx: Receiver<AppEvent>) {
        let start_time: Instant = Instant::now();

        // recv() parks the thread with zero CPU usage until an event arrives
        while let Ok(event) = rx.recv() {
            match event {
                AppEvent::LoadComplete {
                    desire_line_count: traj_count,
                    correlation_percent,
                } => {
                    println!(
                        "[LOG] LOAD COMPLETED at {:?} — {} trajectories loaded, correlation: {:.2}%.",
                        start_time.elapsed(),
                        traj_count,
                        correlation_percent
                    );
                }
                AppEvent::ComputationStart {
                    computation_type,
                    max_progress,
                } => {
                    println!(
                        "[LOG] COMPUTATION STARTED at {:?} — {:?} with {} total steps.",
                        start_time.elapsed(),
                        computation_type,
                        max_progress,
                    );
                }
                AppEvent::ComputationProgress {
                    computation_type,
                    increment_progress,
                } => {
                    println!(
                        "[LOG] {:?} progress: +{} steps at {:?}.",
                        computation_type,
                        increment_progress,
                        start_time.elapsed()
                    );
                }
                AppEvent::ComputationComplete { computation_type } => {
                    println!(
                        "[LOG] {:?} COMPLETED at {:?}.",
                        computation_type,
                        start_time.elapsed()
                    );
                }
                AppEvent::Error(msg) => {
                    eprintln!("[LOG][ERROR] {}", msg);
                }
            }
        }

        println!("[LOG] Logger shutting down.");
    }
}
