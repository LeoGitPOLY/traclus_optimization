use crate::logger::Logger;
use crate::perf_timer::PerfTimer;
use clap::Parser;
use traclusdl_core::io::args::{InterfaceMode, TraclusArgs};
use traclusdl_core::traclusdl_core::TraclusDLCore;
use traclusdl_core::utils::events::event_singleton;

mod logger;
mod perf_timer;

fn main() -> std::io::Result<()> {
    let traclus_args: TraclusArgs = TraclusArgs::parse();
    let main_traclusdl: TraclusDLCore = TraclusDLCore::new();

    TraclusDLCore::build_thread_pool(&traclus_args, false);

    // Start the logger thread or perf timer thread
    let handle = match traclus_args.interface_mode {
        InterfaceMode::Logger => Some(Logger::start()),
        InterfaceMode::PerfTimer => Some(PerfTimer::start()),
        _ => None,
    };

    main_traclusdl.run_full_traclus(traclus_args);

    // Correctly stop events and logger thread
    event_singleton::shutdown();
    if let Some(handle) = handle {
        handle.join().expect("Thread panicked");
    }

    Ok(())
}
