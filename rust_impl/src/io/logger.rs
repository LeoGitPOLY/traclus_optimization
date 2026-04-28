// logger.rs - Event subscriber that prints AppEvents to stdout
//
// The logger runs on its own dedicated std::thread
// CPU usage is kept low with zero busy-wait — the thread parks completely between events.

use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::thread::{self, JoinHandle};
use std::time::Instant;

use crate::utils::events::app_events::AppEvent;
use crate::utils::events::event_singleton::subscribe as singleton_subscribe;

struct PerfRecord {
    label: &'static str,
    elapsed_ms: f64,
    instances: usize,
    parent: Option<&'static str>,
    children: Vec<&'static str>,
}
impl PerfRecord {
    fn new(label: &'static str, parent: Option<&'static str>) -> Self {
        Self {
            label,
            elapsed_ms: 0.0,
            instances: 0,
            parent,
            children: Vec::new(),
        }
    }
}

struct ActivePerfTimer {
    instant: Instant,
    parent: Option<&'static str>,
}
impl ActivePerfTimer {
    fn new(instant: Instant, parent: Option<&'static str>) -> Self {
        Self { instant, parent }
    }
}
pub struct Logger;

impl Logger {
    /// Spawn the logger thread.
    /// `rx` is the Receiver obtained from EventBus::subscribe().
    pub fn start() -> JoinHandle<()> {
        let rx: Receiver<AppEvent> = singleton_subscribe();
        thread::Builder::new()
            .name("traclus-logger".to_string())
            .spawn(move || Self::run(rx))
            .expect("failed to spawn logger thread")
    }

    fn run(rx: Receiver<AppEvent>) {
        let start_time: Instant = Instant::now();

        let mut active_timers: Vec<(&'static str, ActivePerfTimer)> = Vec::new();
        let mut records: HashMap<&'static str, PerfRecord> = HashMap::new();
        let mut root_order: Vec<&'static str> = Vec::new();

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
                AppEvent::PrintInfo { messages } => {
                    for message in messages {
                        println!("[LOG] {}", message);
                    }
                }
                AppEvent::PerfTimer {
                    event_label,
                    exact_instant,
                    is_start,
                } => {
                    if is_start {
                        // Starting timer: get parent task + push onto active stack
                        let parent: Option<&str> = active_timers.last().map(|(l, _)| *l);
                        active_timers
                            .push((event_label, ActivePerfTimer::new(exact_instant, parent)));
                    } else {
                        // Ending timer: pop from active stack, calculate elapsed, and update records
                        if let Some(pos) = active_timers.iter().position(|(l, _)| *l == event_label)
                        {
                            let (_, timer) = active_timers.remove(pos);
                            let elapsed_ms: f64 =
                                exact_instant.duration_since(timer.instant).as_secs_f64() * 1000.0;

                            let record: &mut PerfRecord =
                                records.entry(event_label).or_insert_with(|| {
                                    if timer.parent.is_none() {
                                        root_order.push(event_label);
                                    }
                                    PerfRecord::new(event_label, timer.parent)
                                });
                            record.elapsed_ms += elapsed_ms;
                            record.instances += 1;

                            if let Some(parent_label) = timer.parent {
                                let parent_record: &mut PerfRecord =
                                    records.entry(parent_label).or_insert_with(|| {
                                        root_order.push(parent_label);
                                        PerfRecord::new(parent_label, None)
                                    });
                                if !parent_record.children.contains(&event_label) {
                                    parent_record.children.push(event_label);
                                }
                            }
                        }
                    }
                }

                AppEvent::Error(msg) => {
                    eprintln!("[LOG][ERROR] {}", msg);
                }
            }
        }
        Self::print_summary(&records, &root_order);
    }

    fn print_summary(records: &HashMap<&'static str, PerfRecord>, root_order: &[&'static str]) {
        println!("\n[PERF] ───────────── SUMMARY ─────────────");

        let total: f64 = root_order
            .iter()
            .filter_map(|l| records.get(l))
            .map(|r| r.elapsed_ms)
            .sum();

        for &label in root_order {
            Self::print_record(records, label, total, 0);
        }

        println!("[PERF] ───────────────────────────────────");
        println!("[PERF] {:<30}: {:>10.3} ms", "TOTAL", total);
    }

    fn print_record(
        records: &HashMap<&'static str, PerfRecord>,
        label: &'static str,
        total: f64,
        depth: usize,
    ) {
        let Some(record) = records.get(label) else {
            return;
        };

        let indent = "  ".repeat(depth);
        let label_width = 30usize.saturating_sub(depth * 2);
        let percent_of_total = (record.elapsed_ms / total) * 100.0;
        let instance_tag = if record.instances > 1 {
            format!(" ×{}", record.instances)
        } else {
            String::new()
        };

        if depth == 0 {
            println!(
                "[PERF] {}{:<width$}: {:>10.3} ms ({:>6.2}% of total){}",
                indent,
                record.label,
                record.elapsed_ms,
                percent_of_total,
                instance_tag,
                width = label_width,
            );
        } else {
            let parent_ms = record
                .parent
                .and_then(|p| records.get(p))
                .map(|p| p.elapsed_ms)
                .unwrap_or(record.elapsed_ms);
            let percent_of_parent = (record.elapsed_ms / parent_ms) * 100.0;
            println!(
                "[PERF] {}{:<width$}: {:>10.3} ms ({:>6.2}% of parent){}",
                indent,
                record.label,
                record.elapsed_ms,
                percent_of_parent,
                instance_tag,
                width = label_width,
            );
        }

        // Recurse into children.
        if !record.children.is_empty() {
            let children_ms: f64 = record
                .children
                .iter()
                .filter_map(|l| records.get(l))
                .map(|r| r.elapsed_ms)
                .sum();

            for &child_label in &record.children {
                Self::print_record(records, child_label, total, depth + 1);
            }

            // "Other" = time the parent spent outside of any tracked child.
            let other_ms = record.elapsed_ms - children_ms;
            if other_ms > 0.001 {
                let indent_child = "  ".repeat(depth + 1);
                let child_width = 30usize.saturating_sub((depth + 1) * 2);
                let other_pct_parent = (other_ms / record.elapsed_ms) * 100.0;
                println!(
                    "[PERF] {}{:<width$}: {:>10.3} ms ({:>6.2}% of parent)",
                    indent_child,
                    "(other)",
                    other_ms,
                    other_pct_parent,
                    width = child_width,
                );
            }
        }
    }
}
