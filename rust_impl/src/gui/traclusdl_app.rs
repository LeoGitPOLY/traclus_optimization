// traclus_app.rs - Main application state and entry point

use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

use eframe::egui;

use crate::algorithms::main_traclusdl::MainTraclusDL;
use crate::gui::style::*;
use crate::gui::view_model::ViewModel;

use crate::gui::app_events::AppEvent;
use crate::io::args::TraclusArgs;
use crate::utils::gui_parallel_runner::GuiParallelRunner;

// ─────────────────────────────────────────────
// Application State
// ─────────────────────────────────────────────

pub struct TraclusDLApp {
    pub vm: ViewModel,

    pub detected_cpus: usize,
    pub output_text: String,

    pub main_traclus: Arc<Mutex<MainTraclusDL>>,
    pub runner: GuiParallelRunner,

    event_rx: Receiver<AppEvent>,
}

impl TraclusDLApp {
    // TraclusDLApp::new is private — construction only via start_gui
    fn new(main_traclusdl: MainTraclusDL) -> Self {
        let main_traclus: Arc<Mutex<MainTraclusDL>> = Arc::new(Mutex::new(main_traclusdl));
        let event_rx: Receiver<AppEvent> = main_traclus.lock().unwrap().event.subscribe();

        Self {
            vm: ViewModel::default(),
            detected_cpus: num_cpus_detected(),
            output_text: String::new(),

            main_traclus,
            runner: GuiParallelRunner::new(),
            event_rx,
        }
    }

    // ─────────────────────────────────────────────
    // GUI button actions
    // ─────────────────────────────────────────────
    pub fn on_browse_done(&mut self, path: PathBuf) {
        self.vm.input_file_path = path.display().to_string();
        self.vm.input_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        self.vm.num_dl = 0;
        self.vm.percent_correlation = 0.0;

        let infile: String = self.vm.input_file_path.clone();
        self.launch(move |t| {
            t.load_raw_storage(&infile);
        });
    }

    pub fn on_start_computation(&mut self) {
        let args: TraclusArgs = TraclusArgs::default();

        self.launch(move |t| {
            t.run_clustering(&args);
        });
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
        match event {
            AppEvent::LoadComplete {
                traj_count,
                correlation_percent,
            } => {
                self.vm.num_dl = traj_count;
                self.vm.percent_correlation = correlation_percent;
            }

            AppEvent::ComputationClusteringProgress { num_traj_done } => {
                self.output_text +=
                    &format!("Clustering progress: {} trajectories done.", num_traj_done);
            }

            AppEvent::ComputationComplete {
                total_corridors,
                total_seg,
                total_seg_outside_corridor,
            } => {
                self.output_text += &format!(
                    "Computation complete: {} corridors, {} segments, {} segments outside corridor.",
                    total_corridors, total_seg, total_seg_outside_corridor
                );
            }

            AppEvent::Error(msg) => {
                self.output_text = format!("Error: {}", msg);
            }
        }
    }

    /// Launches a task on the worker thread via GuiParallelRunner.
    pub fn launch<F>(&mut self, task: F)
    where
        F: FnOnce(&mut MainTraclusDL) + Send + 'static,
    {
        self.runner.try_run(Arc::clone(&self.main_traclus), task);
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

// ─────────────────────────────────────────────
// GUI Entry Point
// ─────────────────────────────────────────────

pub fn start_gui(main_traclusdl: MainTraclusDL) {
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
        Box::new(|_cc| Box::new(TraclusDLApp::new(main_traclusdl))),
    )
    .unwrap();
}
