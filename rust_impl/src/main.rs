use crate::clustering::main_traclusdl::MainTraclusDL;
use crate::gui::traclusdl_app::start_gui;
use crate::io::args::{InterfaceMode, TraclusArgs};
use crate::io::logger::Logger;

use clap::Parser;
use std::thread::available_parallelism;

mod clustering;
mod gui;
mod io;
mod utils;

/// Returns how many threads Rayon should use for computation.
/// Reserves CPUs for the UI threads that will be active.
fn build_thread_pool(args: &TraclusArgs) -> usize {
    let available: usize = available_parallelism().map(|n| n.get()).unwrap_or(2).max(1);

    let reserved: usize = match args.interface_mode {
        InterfaceMode::Gui => 1,          // 1 CPU for the GUI thread
        InterfaceMode::Logger => 1,       // 1 CPU for the logger thread
        InterfaceMode::GuiAndLogger => 2, // 1 CPU each for GUI + logger
        InterfaceMode::Performance => 0,  // no reservation — all CPUs to computation
    };

    let computation: usize = available.saturating_sub(reserved).max(1);
    rayon::ThreadPoolBuilder::new()
        .num_threads(computation)
        .build_global()
        .expect("Failed to build Rayon thread pool");

    println!(
        "Available CPUs: {}, reserved for UI/Logger: {}, used for computation: {}",
        available, reserved, computation
    );

    computation
}
// ─────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────

fn main() -> std::io::Result<()> {
    let traclus_args: TraclusArgs = TraclusArgs::parse();
    let main_traclusdl: MainTraclusDL = MainTraclusDL::new();

    build_thread_pool(&traclus_args);

    // Subscribe all subscribers
    match traclus_args.interface_mode {
        InterfaceMode::Logger | InterfaceMode::GuiAndLogger => {
            Logger::start();
        }
        _ => {}
    }

    // Route to the appropriate front-end
    match traclus_args.interface_mode {
        InterfaceMode::Gui | InterfaceMode::GuiAndLogger => {
            start_gui(traclus_args, main_traclusdl);
        }
        InterfaceMode::Logger | InterfaceMode::Performance => {
            main_traclusdl.run_full_traclus(traclus_args);
        }
    }

    Ok(())
}
