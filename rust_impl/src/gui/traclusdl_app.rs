// traclus_app.rs - Main application state and entry point

use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use eframe::egui;

use crate::clustering::main_traclusdl::MainTraclusDL;
use crate::gui::app_events::AppEvent;
use crate::gui::event_singleton::subscribe;
use crate::gui::style::*;
use crate::gui::view_model::ViewModel;
use crate::io::args::TraclusArgs;
use crate::utils::gui_parallel_runner::{GuiParallelRunner, StopFlag};

// ─────────────────────────────────────────────
// Application State
// ─────────────────────────────────────────────

pub struct TraclusDLApp {
    pub vm: Vec<ViewModel>,
    pub current_selected_vm: usize,

    pub detected_cpus: usize,

    pub main_traclus: Arc<Mutex<MainTraclusDL>>,
    pub runner: GuiParallelRunner,

    event_rx: Receiver<AppEvent>,
}

impl TraclusDLApp {
    // TraclusDLApp::new is private — construction only via start_gui
    fn new(args: TraclusArgs, main_traclusdl: MainTraclusDL) -> Self {
        let main_traclus: Arc<Mutex<MainTraclusDL>> = Arc::new(Mutex::new(main_traclusdl));
        let event_rx: Receiver<AppEvent> = subscribe();

        Self {
            vm: vec![ViewModel::new(args)],
            current_selected_vm: 0,
            detected_cpus: num_cpus_detected(),

            main_traclus,
            runner: GuiParallelRunner::new(),
            event_rx,
        }
    }

    // ─────────────────────────────────────────────
    // GUI button actions
    // ─────────────────────────────────────────────
    pub fn on_browse_done(&mut self, path: PathBuf) {
        let vm: &mut ViewModel = self.current_vm();
        vm.args_selected.file = path.display().to_string();
        vm.args_buffer.input_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        vm.num_dl = 0;
        vm.percent_correlation = 0.0;
        vm.output.clear();

        let args: TraclusArgs = vm.args_selected.clone();
        vm.args_when_loaded = args.clone();
        self.launch(move |t, stop| {
            t.load_raw_storage(&args, stop);
        });
    }

    pub fn on_start_computation(&mut self) {
        let args: TraclusArgs = self.current_vm().args_selected.clone();
        let args_when_loaded: TraclusArgs = self.current_vm().args_when_loaded.clone();
        let needs_reload: bool = args != args_when_loaded;

        self.current_vm().output.clear();
        self.current_vm().output += "=== Starting clustering computation ===\n";
        self.current_vm().output += &args.print_small_summary();
        self.current_vm().output += "\n";

        self.launch(move |t, stop| {
            if needs_reload {
                t.load_raw_storage(&args, stop.clone());
            }
            t.run_clustering(&args, stop);
        });
    }
    pub fn on_generate_outputs(&mut self) {
        let vm: &mut ViewModel = self.current_vm();
        let args: TraclusArgs = vm.args_selected.clone();

        self.launch(move |t, stop| {
            t.generate_outputs(&args, stop);
        });
    }

    pub fn on_stop_computation(&mut self) {
        self.runner.stop();
        self.current_vm().output += " -> STOPPED \n";
    }

    pub fn on_plus_vm(&mut self) {
        let vm: &mut ViewModel = self.current_vm();
        vm.error_popup = Some("Functionality not implemented yet. Coming soon!".to_string());
        // app.vm.push(crate::gui::view_model::ViewModel::default());
    }

    // ─────────────────────────────────────────────
    // Events handling
    // ─────────────────────────────────────────────

    /// Drains all pending events from the channel and updates GUI state.
    pub fn drain_events(&mut self) {
        // try_recv is non-blocking — returns Err(Empty) immediately when nothing is queued
        while let Ok(event) = self.event_rx.try_recv() {
            self.handle_event(event);
        }
    }

    fn handle_event(&mut self, event: AppEvent) {
        let vm: &mut ViewModel = self.current_vm();

        match event {
            AppEvent::LoadComplete {
                desire_line_count,
                correlation_percent,
            } => {
                vm.num_dl = desire_line_count;
                vm.percent_correlation = correlation_percent;
                vm.input_name = vm.args_buffer.input_name.clone();
            }

            AppEvent::ComputationStart {
                computation_type,
                max_progress,
            } => {
                vm.total_to_compute = max_progress;
                vm.num_computed = 0;
                vm.start_time_computation = Instant::now();
                vm.computation_type = computation_type.clone();

                vm.output += &format!("Started {:?} computation ", computation_type,)
                    .trim_end_matches('\n')
                    .to_string();
            }

            AppEvent::ComputationProgress {
                computation_type: _,
                increment_progress,
            } => {
                vm.num_computed += increment_progress;
                vm.estimated_time_total = estimated_time_total(
                    vm.start_time_computation,
                    vm.num_computed as f64 / vm.total_to_compute as f64,
                );
            }

            AppEvent::ComputationComplete { computation_type } => {
                vm.num_computed = vm.total_to_compute;
                let elapsed = vm.start_time_computation.elapsed().as_secs_f64() as u64;

                if computation_type == vm.computation_type {
                    vm.output += &format!(" -> Completed ({}s) \n", elapsed);
                }
            }

            AppEvent::PrintInfo { messages } => {
                vm.output += "\n";
                for message in messages {
                    vm.output += &format!("{}\n", message);
                }
                vm.output += "\n";
            }

            AppEvent::Error(msg) => {
                vm.error_popup = Some(msg.to_string());
            }
        }
    }

    /// Launches a task on the worker thread via GuiParallelRunner.
    pub fn launch<F>(&mut self, task: F)
    where
        F: FnOnce(&mut MainTraclusDL, StopFlag) + Send + 'static,
    {
        self.runner.try_run(Arc::clone(&self.main_traclus), task);
    }

    /// Returns a mutable reference to the currently selected ViewModel.
    pub fn current_vm(&mut self) -> &mut ViewModel {
        &mut self.vm[self.current_selected_vm]
    }
}

// ─────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────

fn num_cpus_detected() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}
fn estimated_time_total(start: std::time::Instant, progress_percent: f64) -> f64 {
    let real_elapsed: f64 = start.elapsed().as_secs_f64();

    if progress_percent <= 0.1 {
        return 0.0;
    }

    let raw_estimate: f64 = real_elapsed / progress_percent;

    // Multiplier fades from `initial_boost` at 10% progress down to 1.0 at `fade_until`
    // Before fade_until : estimate is inflated   → shows a safe "high" value early on
    // After  fade_until : raw estimate takes over → reflects reality
    let initial_boost: f64 = 2.0; // how much to overestimate at the start
    let fade_until: f64 = 0.5; // progress at which multiplier fully reaches 1.0

    let multiplier: f64 = if progress_percent >= fade_until {
        1.0
    } else {
        // Linear interpolation from initial_boost → 1.0 as progress goes 0.1 → fade_until
        let t = (progress_percent - 0.1) / (fade_until - 0.1);
        initial_boost + t * (1.0 - initial_boost)
    };

    raw_estimate * multiplier
}

// ─────────────────────────────────────────────
// GUI Entry Point
// ─────────────────────────────────────────────

pub fn start_gui(args: TraclusArgs, main_traclusdl: MainTraclusDL) {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_resizable(false)
            .with_maximize_button(false),
        ..Default::default()
    };

    eframe::run_native(
        "Traclus_DL - Rust Implementation",
        options,
        Box::new(|_cc| Box::new(TraclusDLApp::new(args, main_traclusdl))),
    )
    .unwrap();
}
