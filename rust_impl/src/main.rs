mod algorithms;
mod clustering;
mod geometry;
mod gui;
mod io;
mod storage;

use crate::algorithms::main_traclusdl::run_traclus;
use crate::gui::traclus_app::start_gui;
use crate::io::args::TraclusArgs;

use clap::Parser;

fn main() -> std::io::Result<()> {
    let traclus_args: TraclusArgs = TraclusArgs::parse();

    if traclus_args.gui {
        start_gui();
    } else {
        run_traclus(traclus_args)
    }

    Ok(())
}
