// view_model.rs - Data bound to the GUI fields (form state)

use std::time::Instant;

use crate::io::args::TraclusArgs;

// ─────────────────────────────────────────────
// ArgsBuffer
// ─────────────────────────────────────────────

pub struct ArgsBuffer {
    pub max_dist: String,
    pub min_density: String,
    pub max_angle: String,
    pub segment_size: String,
}

impl ArgsBuffer {
    pub fn from_args(args: &TraclusArgs) -> Self {
        Self {
            max_dist: args.max_dist.to_string(),
            min_density: args.min_density.to_string(),
            max_angle: args.max_angle.to_string(),
            segment_size: args.segment_size.to_string(),
        }
    }
}

// ─────────────────────────────────────────────
// ViewModel
// ─────────────────────────────────────────────

pub struct ViewModel {
    pub args: TraclusArgs,
    pub args_buffer: ArgsBuffer,

    // Input file info section
    pub input_name: String,
    pub num_dl: usize,
    pub percent_correlation: f64,

    // Computation info section
    pub num_computation_threads: usize,
    pub num_clustered_traj: usize,
    pub num_total_traj: usize,
    pub start_time_computation: Instant,
    pub estimated_time_remaining: f64,

    // Output section
    pub output: String,

    // Error section
    pub error_popup: Option<String>,
}
impl ViewModel {
    pub fn new(args: TraclusArgs) -> Self {
        let args_buffer: ArgsBuffer = ArgsBuffer::from_args(&args);
        Self {
            args,
            args_buffer,

            input_name: String::new(),
            num_dl: 0,
            percent_correlation: 0.0,

            num_computation_threads: 0,
            num_total_traj: 0,
            num_clustered_traj: 0,
            start_time_computation: Instant::now(),
            estimated_time_remaining: 0.0,

            output: String::new(),
            error_popup: None,
        }
    }
}

impl Default for ViewModel {
    fn default() -> Self {
        Self::new(TraclusArgs::default())
    }
}
