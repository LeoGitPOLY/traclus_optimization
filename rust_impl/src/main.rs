mod algorithms;
mod clustering;
mod geometry;
mod io;
mod storage;

use crate::algorithms::parallel_rayon_traclusdl::ParallelRayonTraclusDL;
use crate::storage::clustered_trajectories::ClusteredTrajectories;
use crate::storage::raw_trajectories::RawTrajectories;

use crate::algorithms::base_traclusdl::TraclusAlgorithm;
use crate::algorithms::serial_traclusdl::SerialTraclusDL;

use crate::io::input_loader::parse_input_data;
use crate::io::output_writer::{
    SegmentOutputFormat, generate_corridor_file, generate_segment_file,
};
use crate::io::traclus_args::{ExecutionMode, TraclusArgs};

use clap::Parser;

fn get_proper_algorithm(args: &TraclusArgs) -> Box<dyn TraclusAlgorithm> {
    match args.mode {
        ExecutionMode::Serial => Box::new(SerialTraclusDL::new(args.clone())),
        ExecutionMode::ParallelRayon => Box::new(ParallelRayonTraclusDL::new(args.clone())),
    }
}

fn main() -> std::io::Result<()> {
    let args: TraclusArgs = TraclusArgs::parse();

    let raw_storage: RawTrajectories = parse_input_data(&args);
    let mut clust_storage: ClusteredTrajectories = ClusteredTrajectories::new();

    let clustering_algorithm: Box<dyn TraclusAlgorithm> = get_proper_algorithm(&args);
    clustering_algorithm.db_scan_clustering(&raw_storage, &mut clust_storage);

    generate_corridor_file(&args, &clust_storage);
    generate_segment_file(&args, &clust_storage, SegmentOutputFormat::NewTraclus);
    generate_segment_file(&args, &clust_storage, SegmentOutputFormat::OldTraclus);

    Ok(())
}
