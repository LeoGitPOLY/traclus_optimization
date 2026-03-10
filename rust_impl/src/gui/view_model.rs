// view_model.rs - Data bound to the GUI fields (form state)

use crate::io::args::TraclusArgs;
use crate::io::args_config::get_param_configs;

// ─────────────────────────────────────────────
// ArgsBuffer
// ─────────────────────────────────────────────

pub struct ArgsBuffer {
    pub max_dist: String,
    pub min_density: String,
    pub max_angle: String,
    pub segment_size: String,
}

impl Default for ArgsBuffer {
    fn default() -> Self {
        // Initialise buffers from the same defaults as TraclusArgs
        let cfg = get_param_configs();
        Self {
            max_dist: cfg.max_dist.default.to_string(),
            min_density: cfg.min_density.default.to_string(),
            max_angle: cfg.max_angle.default.to_string(),
            segment_size: cfg.segment_size.default.to_string(),
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

    // Output section
    pub output: String,
}
impl ViewModel {
    pub fn new(args: TraclusArgs) -> Self {
        Self {
            args,
            args_buffer: ArgsBuffer::default(),

            input_name: String::new(),
            num_dl: 0,
            percent_correlation: 0.0,

            output: String::new(),
        }
    }
}

impl Default for ViewModel {
    fn default() -> Self {
        Self {
            args: TraclusArgs::default(),
            args_buffer: ArgsBuffer::default(),

            input_name: String::new(),
            num_dl: 0,
            percent_correlation: 0.0,

            output: String::new(),
        }
    }
}
