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
                    traj_count,
                    correlation_percent,
                } => {
                    println!(
                        "[LOG] LOAD COMPLETED at {:?} — {} trajectories loaded, correlation: {:.2}%.",
                        start_time.elapsed(),
                        traj_count,
                        correlation_percent
                    );
                }

                AppEvent::ComputationClusteringProgress { num_traj_done } => {
                    println!(
                        "[LOG] COMPUTATION CLUSTERING PROGRESS at {:?} — {} trajectories done.",
                        start_time.elapsed(),
                        num_traj_done
                    );
                }

                AppEvent::ComputationComplete {
                    total_corridors,
                    total_seg,
                    total_seg_outside_corridor,
                } => {
                    println!(
                        "[LOG] COMPUTATION COMPLETE at {:?} — {} corridors, {} segments, {} segments outside corridor.",
                        start_time.elapsed(),
                        total_corridors,
                        total_seg,
                        total_seg_outside_corridor
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
