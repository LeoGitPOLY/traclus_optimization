use super::storage::clustered_trajectories::ClusteredTrajectories;
use super::storage::raw_trajectories::RawTrajectories;
use crate::gui::app_events::{AppError, AppEvent, ComputationType};

use crate::gui::event_singleton::{emit, emit_error};
use crate::io::args::{ExecutionMode, TraclusArgs};
use crate::io::input_loader::parse_input_data;
use crate::io::output_writer::{SegOutFormat, generate_corridor_file, generate_segment_file};
use crate::utils::gui_parallel_runner::StopFlag;
use crate::utils::statistic::directional_correlation;

use super::algorithms::base_traclusdl::TraclusAlgorithm;
use super::algorithms::parallel_rayon_traclusdl::ParallelRayonTraclusDL;
use super::algorithms::serial_traclusdl::SerialTraclusDL;

pub struct MainTraclusDL {
    raw_storage: Option<RawTrajectories>,
    clust_storage: Option<ClusteredTrajectories>,
}

impl MainTraclusDL {
    pub fn new() -> Self {
        Self {
            raw_storage: None,
            clust_storage: None,
        }
    }

    // Loads raw trajectories from disk and stores them.
    pub fn load_raw_storage(&mut self, args: &TraclusArgs, _: StopFlag) {
        self.raw_storage = parse_input_data(&args);
        self.clust_storage = None;

        if self.raw_storage.is_none() {
            return;
        }

        // Emit information about the loaded data
        emit(AppEvent::LoadComplete {
            desire_line_count: self.raw_storage.as_ref().unwrap().get_total_trajectories(),
            correlation_percent: directional_correlation(self.raw_storage.as_ref().unwrap()),
        });
    }

    // Runs the clustering algorithm on the currently loaded raw storage and stores the clustered result.
    pub fn run_clustering(&mut self, args: &TraclusArgs, stop: StopFlag) {
        if self.raw_storage.is_none() {
            emit_error(AppError::NoRawStorage);
            return;
        }
        self.clust_storage = None;
        let raw_storage: &RawTrajectories = self.raw_storage.as_ref().unwrap();
        let mut clust_storage: ClusteredTrajectories = ClusteredTrajectories::new(&args);

        let mut clustering_algorithm: Box<dyn TraclusAlgorithm> = Self::get_proper_algorithm(args);
        clustering_algorithm.set_stop_flag(stop);
        let result: bool = clustering_algorithm.db_scan_clustering(raw_storage, &mut clust_storage);
        if result {
            emit(AppEvent::PrintInfo {
                messages: clust_storage.get_summary(),
            });
            self.clust_storage = Some(clust_storage);
        }
    }

    // Writes corridor and segment output files from the current clustered storage.
    pub fn generate_outputs(&mut self, _: &TraclusArgs, _: StopFlag) {
        if self.clust_storage.is_none() {
            emit_error(AppError::NoClustStorage);
            return;
        }

        emit(AppEvent::ComputationStart {
            computation_type: ComputationType::CreateOutputs,
            max_progress: 1, // not used for output generation
        });

        let clust_storage: &ClusteredTrajectories = self.clust_storage.as_ref().unwrap();
        let args: &TraclusArgs = &clust_storage.args_snapshot;

        generate_corridor_file(args, clust_storage);
        generate_segment_file(args, clust_storage, SegOutFormat::NewTraclus);
        emit(AppEvent::ComputationComplete {
            computation_type: ComputationType::CreateOutputs,
        });
    }

    /// Commmand line entry point for running the full TraclusDL algorithm
    /// No GUI involved, No overhead of statistics, just pure algorithm execution
    pub fn run_full_traclus(&self, args: TraclusArgs) {
        let raw_storage: RawTrajectories =
            parse_input_data(&args).expect("Failed to parse input data");

        let mut clust_storage: ClusteredTrajectories = ClusteredTrajectories::new(&args);
        let clustering_algorithm: Box<dyn TraclusAlgorithm> = Self::get_proper_algorithm(&args);
        clustering_algorithm.db_scan_clustering(&raw_storage, &mut clust_storage);

        generate_corridor_file(&args, &clust_storage);
        generate_segment_file(&args, &clust_storage, SegOutFormat::NewTraclus);
        generate_segment_file(&args, &clust_storage, SegOutFormat::OldTraclus);
    }

    fn get_proper_algorithm(args: &TraclusArgs) -> Box<dyn TraclusAlgorithm> {
        match args.mode {
            ExecutionMode::Serial => Box::new(SerialTraclusDL::new(args.clone())),
            ExecutionMode::ParallelRayon => Box::new(ParallelRayonTraclusDL::new(args.clone())),
        }
    }
}
