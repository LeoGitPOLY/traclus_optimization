use super::storage::clustered_trajectories::ClusteredTrajectories;
use super::storage::raw_trajectories::RawTrajectories;
use crate::gui::app_events::{AppError, AppEvent, ComputationEvent};

use crate::io::args::{ExecutionMode, TraclusArgs};
use crate::io::input_loader::parse_input_data;
use crate::io::output_writer::{
    SegmentOutputFormat, generate_corridor_file, generate_segment_file,
};
use crate::utils::gui_parallel_runner::StopFlag;

use super::algorithms::base_traclusdl::TraclusAlgorithm;
use super::algorithms::parallel_rayon_traclusdl::ParallelRayonTraclusDL;
use super::algorithms::serial_traclusdl::SerialTraclusDL;

pub struct MainTraclusDL {
    raw_storage: Option<RawTrajectories>,
    clust_storage: Option<ClusteredTrajectories>,
    pub event: ComputationEvent,
}

impl MainTraclusDL {
    pub fn new() -> Self {
        Self {
            raw_storage: None,
            clust_storage: None,
            event: ComputationEvent::new(),
        }
    }

    // Loads raw trajectories from disk and stores them.
    pub fn load_raw_storage(&mut self, args: &TraclusArgs, _: StopFlag) {
        self.raw_storage = parse_input_data(&args, &mut self.event);

        if self.raw_storage.is_none() {
            return;
        }

        // Emit information about the loaded data
        self.event.emit(AppEvent::LoadComplete {
            desire_line_count: self.raw_storage.as_ref().unwrap().get_total_trajectories(),
            correlation_percent: 10.0, // TODO: compute actual correlation
        });
    }

    // Runs the clustering algorithm on the currently loaded raw storage and stores the clustered result.
    pub fn run_clustering(&mut self, args: &TraclusArgs, stop: StopFlag) {
        if self.raw_storage.is_none() {
            self.event.emit_error(AppError::NoRawStorage);
            return;
        }

        let raw_storage: &RawTrajectories = self.raw_storage.as_ref().unwrap();
        let mut clust_storage: ClusteredTrajectories = ClusteredTrajectories::new();

        let mut clustering_algorithm: Box<dyn TraclusAlgorithm> = Self::get_proper_algorithm(args);
        clustering_algorithm.set_stop_flag(stop);
        clustering_algorithm.db_scan_clustering(raw_storage, &mut clust_storage, &mut self.event);

        self.clust_storage = Some(clust_storage);
    }

    // Writes corridor and segment output files from the current clustered storage.
    pub fn generate_outputs(&self, args: &TraclusArgs, _: StopFlag) {
        if let Some(clust) = &self.clust_storage {
            generate_corridor_file(args, clust);
            generate_segment_file(args, clust, SegmentOutputFormat::NewTraclus);
            generate_segment_file(args, clust, SegmentOutputFormat::OldTraclus);
        }
    }

    /// Commmand line entry point for running the full TraclusDL algorithm
    /// No GUI involved, No overhead of statistics, just pure algorithm execution
    pub fn run_full_traclus(&self, args: TraclusArgs) {
        let mut empty_event = ComputationEvent::new();
        let raw_storage: RawTrajectories =
            parse_input_data(&args, &mut empty_event).expect("Failed to parse input data");

        let mut clust_storage: ClusteredTrajectories = ClusteredTrajectories::new();
        let clustering_algorithm: Box<dyn TraclusAlgorithm> = Self::get_proper_algorithm(&args);
        clustering_algorithm.db_scan_clustering(&raw_storage, &mut clust_storage, &mut empty_event);

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
