use rayon::ThreadPool;

use super::storage::clustered_trajectories::ClusteredTrajectories;
use super::storage::raw_trajectories::RawTrajectories;
use crate::gui::app_events::{AppError, AppEvent, ComputationEvent};

use crate::io::args::{ExecutionMode, TraclusArgs};
use crate::io::input_loader::parse_input_data;
use crate::io::output_writer::{
    SegmentOutputFormat, generate_corridor_file, generate_segment_file,
};

use super::algorithms::base_traclusdl::TraclusAlgorithm;
use super::algorithms::parallel_rayon_traclusdl::ParallelRayonTraclusDL;
use super::algorithms::serial_traclusdl::SerialTraclusDL;


pub struct MainTraclusDL {
    raw_storage: Option<RawTrajectories>,
    clust_storage: Option<ClusteredTrajectories>,

    rayon_pool: ThreadPool,
    pub event: ComputationEvent,
}

impl MainTraclusDL {
    pub fn new(num_computation_threads: usize) -> Self {
        let rayon_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_computation_threads.max(1))
            .build()
            .expect("failed to build Rayon thread pool");

        Self {
            raw_storage: None,
            clust_storage: None,

            rayon_pool,
            event: ComputationEvent::new(),
        }
    }

    // Loads raw trajectories from disk and stores them.
    pub fn load_raw_storage(&mut self, args: &TraclusArgs) {
        self.raw_storage = Some(parse_input_data(&args));

        // Emit information about the loaded data
        self.event.emit(AppEvent::LoadComplete {
            traj_count: self.raw_storage.as_ref().unwrap().get_total_trajectories(),
            correlation_percent: 10.0, // TODO: compute actual correlation
        });
    }

    // Runs the clustering algorithm on the currently loaded raw storage and stores the clustered result.
    pub fn run_clustering(&mut self, args: &TraclusArgs) {
        self.event
            .emit(AppEvent::Error(AppError::IoError((format!("{:?}", args)))));

        if self.raw_storage.is_none() {
            self.event.emit(AppEvent::Error(AppError::NoRawStorage));
            return;
        }

        let raw_storage: &RawTrajectories = self.raw_storage.as_ref().unwrap();
        let mut clust_storage: ClusteredTrajectories = ClusteredTrajectories::new();

        let clustering_algorithm: Box<dyn TraclusAlgorithm> = Self::get_proper_algorithm(args);
        clustering_algorithm.db_scan_clustering(raw_storage, &mut clust_storage);

        self.clust_storage = Some(clust_storage);
    }

    // Writes corridor and segment output files from the current clustered storage.
    pub fn generate_outputs(&self, args: &TraclusArgs) {
        if let Some(clust) = &self.clust_storage {
            generate_corridor_file(args, clust);
            generate_segment_file(args, clust, SegmentOutputFormat::NewTraclus);
            generate_segment_file(args, clust, SegmentOutputFormat::OldTraclus);
        }
    }

    /// Commmand line entry point for running the full TraclusDL algorithm
    /// No GUI involved, No overhead of statistics, just pure algorithm execution
    pub fn run_full_traclus(&self, args: TraclusArgs) {
        let raw_storage: RawTrajectories = parse_input_data(&args);
        let mut clust_storage: ClusteredTrajectories = ClusteredTrajectories::new();

        let clustering_algorithm: Box<dyn TraclusAlgorithm> = Self::get_proper_algorithm(&args);
        clustering_algorithm.db_scan_clustering(&raw_storage, &mut clust_storage);

        generate_corridor_file(&args, &clust_storage);
        generate_segment_file(&args, &clust_storage, SegmentOutputFormat::NewTraclus);
        generate_segment_file(&args, &clust_storage, SegmentOutputFormat::OldTraclus);
    }

    fn get_proper_algorithm(args: &TraclusArgs) -> Box<dyn TraclusAlgorithm> {
        match args.mode {
            ExecutionMode::Serial => Box::new(SerialTraclusDL::new(args.clone())),
            ExecutionMode::ParallelRayon => Box::new(ParallelRayonTraclusDL::new(args.clone())),
        }
    }
}
