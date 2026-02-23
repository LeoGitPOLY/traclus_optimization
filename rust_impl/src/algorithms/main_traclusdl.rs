use crate::storage::clustered_trajectories::ClusteredTrajectories;
use crate::storage::raw_trajectories::RawTrajectories;

use crate::io::input_loader::parse_input_data;
use crate::io::output_writer::{
    SegmentOutputFormat, generate_corridor_file, generate_segment_file,
};
use crate::io::args::{ExecutionMode, TraclusArgs};

use crate::algorithms::base_traclusdl::TraclusAlgorithm;
use crate::algorithms::parallel_rayon_traclusdl::ParallelRayonTraclusDL;
use crate::algorithms::serial_traclusdl::SerialTraclusDL;

fn get_proper_algorithm(args: &TraclusArgs) -> Box<dyn TraclusAlgorithm> {
    match args.mode {
        ExecutionMode::Serial => Box::new(SerialTraclusDL::new(args.clone())),
        ExecutionMode::ParallelRayon => Box::new(ParallelRayonTraclusDL::new(args.clone())),
    }
}

pub fn run_traclus(args: TraclusArgs) {
    let raw_storage: RawTrajectories = parse_input_data(&args);
    let mut clust_storage: ClusteredTrajectories = ClusteredTrajectories::new();

    let clustering_algorithm: Box<dyn TraclusAlgorithm> = get_proper_algorithm(&args);
    clustering_algorithm.db_scan_clustering(&raw_storage, &mut clust_storage);

    generate_corridor_file(&args, &clust_storage);
    generate_segment_file(&args, &clust_storage, SegmentOutputFormat::NewTraclus);
    generate_segment_file(&args, &clust_storage, SegmentOutputFormat::OldTraclus);
}
