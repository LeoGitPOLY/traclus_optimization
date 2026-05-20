// logger.rs - Event subscriber that prints AppEvents to stdout
//
// The logger runs on its own dedicated std::thread
// CPU usage stays near zero — the thread is parked while waiting for events.

use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::thread::{self, JoinHandle};
use std::time::Instant;

use crate::utils::events::app_events::AppEvent;
use crate::utils::events::event_singleton::subscribe as singleton_subscribe;

struct PerfRecord {
    display_label: String,
    elapsed_ms: f64,
    instances: usize,
    children: Vec<String>,
}

impl PerfRecord {
    fn new(display_label: String) -> Self {
        Self {
            display_label,
            elapsed_ms: 0.0,
            instances: 0,
            children: Vec::new(),
        }
    }
}

pub struct Logger {
    active_timers: Vec<(String, Instant)>,
    root_elements: Vec<String>,
    all_elements: HashMap<String, PerfRecord>,
}

impl Logger {
    /// Spawn the logger thread.
    /// `rx` is the Receiver returned by EventBus::subscribe().
    pub fn start() -> JoinHandle<()> {
        let rx: Receiver<AppEvent> = singleton_subscribe();

        thread::Builder::new()
            .name("traclus-logger".to_string())
            .spawn(move || Self::run(rx))
            .expect("failed to spawn logger thread")
    }

    fn run(rx: Receiver<AppEvent>) {
        let start_time: Instant = Instant::now();

        let mut logger: Logger = Logger {
            active_timers: Vec::new(),
            root_elements: Vec::new(),
            all_elements: HashMap::new(),
        };

        // recv() parks the thread with zero CPU usage until an event is received
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

                AppEvent::PrintInfo { messages } => {
                    for message in messages {
                        println!("[LOG] {}", message);
                    }
                }

                AppEvent::PerfTimer {
                    event_label,
                    exact_instant,
                    is_start,
                    thread_index,
                } => {
                    let mut full_label: String = event_label.clone();

                    if let Some(tid) = thread_index {
                        full_label = format!("{}(tid:{})", event_label, tid);
                    }

                    if is_start {
                        logger.handle_timer_start(event_label, full_label, exact_instant);
                    } else {
                        logger.handle_timer_end(event_label, full_label, exact_instant);
                    }
                }

                AppEvent::Error(msg) => {
                    eprintln!("[LOG][ERROR] {}", msg);
                }
            }
        }

        logger.print_summary();
    }

    fn handle_timer_start(
        &mut self,
        event_label: String,
        full_label: String,
        exact_instant: Instant,
    ) {
        // Find the current parent timer from the active stack
        // If none exists, this timer is a root-level task
        let parent_name: String = self
            .active_timers
            .iter()
            .rev()
            .find(|(label, _)| label != &event_label)
            .map(|(label, _)| label.clone())
            .unwrap_or_else(|| "".to_string());

        let parent: Option<&mut PerfRecord> = self.all_elements.get_mut(&parent_name);

        if let Some(parent) = parent {
            if !parent.children.contains(&full_label) {
                parent.children.push(full_label.clone());
            }
        } else {
            self.root_elements.push(full_label.clone());
        }

        self.active_timers
            .push((event_label.clone(), exact_instant));

        // Create the record immediately so children can safely reference it
        // Elapsed time and instance count are updated when the timer ends
        if !self.all_elements.contains_key(&full_label) {
            let record_element: PerfRecord = PerfRecord::new(full_label.clone());
            self.all_elements.insert(full_label, record_element);
        }
    }

    fn handle_timer_end(
        &mut self,
        event_label: String,
        full_label: String,
        exact_instant: Instant,
    ) {
        // Remove timer from the active stack, compute elapsed time,
        // then update the associated performance record
        let position: usize = self
            .active_timers
            .iter()
            .position(|(label, _)| *label == event_label)
            .expect("Timer end received for event with no matching start");

        let (_, timer): (String, Instant) = self.active_timers.remove(position);
        let record_element: &mut PerfRecord = self.all_elements.get_mut(&full_label).unwrap();
        let elapsed_ms: f64 = exact_instant.duration_since(timer).as_secs_f64() * 1000.0;

        record_element.elapsed_ms += elapsed_ms;
        record_element.instances += 1;
    }

    fn print_summary(&self) {
        println!("\n[PERF] ──────────────── SUMMARY ────────────────");

        let total_ms: f64 = self
            .root_elements
            .iter()
            .filter_map(|label| self.all_elements.get(label))
            .map(|record| record.elapsed_ms)
            .sum();

        for root in &self.root_elements {
            self.print_parent(root, total_ms, 0);
        }

        println!("[PERF] ─────────────────────────────────────────");
        println!("[PERF] {:<35}: {:>10.3} ms", "TOTAL", total_ms);
    }

    fn print_parent(&self, parent_label: &String, parent_ms: f64, depth: usize) {
        let parent_record: &PerfRecord = self.all_elements.get(parent_label).unwrap();

        Self::print_record(parent_record, parent_ms, depth);

        for child in &parent_record.children {
            self.print_parent(child, parent_record.elapsed_ms, depth + 1);
        }
    }

    fn print_record(record: &PerfRecord, parent_ms: f64, depth: usize) {
        let indent: String = "   ".repeat(depth);

        let label: String = format!("{}{}", indent, record.display_label);

        let instance_tag: String = if record.instances > 1 {
            format!(" ×{}", record.instances)
        } else {
            String::new()
        };

        println!(
            "[PERF] {:<40}: {:>10.3} ms ({:>6.2}%){}",
            label,
            record.elapsed_ms,
            (record.elapsed_ms / parent_ms) * 100.0,
            instance_tag
        );
    }
}
