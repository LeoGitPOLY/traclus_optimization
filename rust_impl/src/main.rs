use crate::algorithms::main_traclusdl::MainTraclusDL;
use crate::gui::app_events::AppEvent;
use crate::gui::traclusdl_app::start_gui;
use crate::io::args::{InterfaceMode, TraclusArgs};
use crate::io::logger::Logger;

use clap::Parser;
use eframe::App;
use std::sync::mpsc::Receiver;
use std::thread::available_parallelism;

mod algorithms;
mod clustering;
mod geometry;
mod gui;
mod io;
mod storage;
mod utils;

// TODO: see if it's the logical or physical cores that limits

/// Returns how many threads Rayon should use for computation.
/// Reserves CPUs for the UI threads that will be active.
fn get_number_of_cpus(args: &TraclusArgs) -> usize {
    let available: usize = available_parallelism().map(|n| n.get()).unwrap_or(2).max(1);

    let reserved: usize = match args.interface_mode {
        InterfaceMode::Gui => 1,          // 1 CPU for the GUI thread
        InterfaceMode::Logger => 1,       // 1 CPU for the logger thread
        InterfaceMode::GuiAndLogger => 2, // 1 CPU each for GUI + logger
        InterfaceMode::Performance => 0,  // no reservation — all CPUs to computation
    };

    let computation: usize = available.saturating_sub(reserved).max(1);

    println!(
        "Available CPUs: {}, reserved for UI: {}, used for computation: {}",
        available, reserved, computation
    );

    computation
}

// ─────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────

fn main() -> std::io::Result<()> {
    let traclus_args: TraclusArgs = TraclusArgs::parse();

    let num_computation_threads: usize = get_number_of_cpus(&traclus_args);
    let mut main_traclusdl: MainTraclusDL = MainTraclusDL::new(num_computation_threads);

    // Subscribe all subscriber
    match traclus_args.interface_mode {
        InterfaceMode::Logger | InterfaceMode::GuiAndLogger => {
            let logger_rx: Receiver<AppEvent> = main_traclusdl.event.subscribe();
            Logger::start(logger_rx);
        }
        _ => {}
    }

    // Route to the appropriate front-end
    match traclus_args.interface_mode {
        InterfaceMode::Gui | InterfaceMode::GuiAndLogger => {
            start_gui(main_traclusdl);
        }
        InterfaceMode::Logger | InterfaceMode::Performance => {
            main_traclusdl.run_full_traclus(traclus_args);
        }
    }

    Ok(())
}
