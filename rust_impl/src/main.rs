mod algorithms;
mod clustering;
mod geometry;
mod io;
mod storage;

use crate::storage::clustered_trajectories::ClusteredTrajectories;
use crate::storage::raw_trajectories::RawTrajectories;

use crate::algorithms::base_traclusdl::TraclusAlgorithm;
use crate::algorithms::serial_traclusdl::SerialTraclusDL;
use crate::io::loader::{parse_input_data, parse_output_data};
use crate::io::traclus_args::TraclusArgs;

use clap::Parser;

fn main() -> std::io::Result<()> {
    let args: TraclusArgs = TraclusArgs::parse();

    let raw_storage: RawTrajectories = parse_input_data(&args);
    let mut clust_storage: ClusteredTrajectories = ClusteredTrajectories::new();

    let clustering_algorithm: SerialTraclusDL = SerialTraclusDL::new(args);
    clustering_algorithm.db_scan_clustering(&raw_storage, &mut clust_storage);
    clustering_algorithm.create_corridors(&mut clust_storage);

    parse_output_data(clustering_algorithm.args(), &clust_storage);

    Ok(())
}
