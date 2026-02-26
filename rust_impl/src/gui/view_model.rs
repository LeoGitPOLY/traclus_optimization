// view_model.rs - Data bound to the GUI fields (form state)
// All fields are public so gui.rs can read and write them directly

// ─────────────────────────────────────────────
// Computing Mode : TODO changed to the real computing enum
// ─────────────────────────────────────────────

#[derive(PartialEq)]
pub enum ComputeMode {
    Serial,
    Parallel,
}

impl Default for ComputeMode {
    fn default() -> Self {
        ComputeMode::Parallel
    }
}

// ─────────────────────────────────────────────
// Parameter Set
// ─────────────────────────────────────────────

pub struct ParameterSet {
    // Committed values — passed to the algorithm
    pub max_angle: i32,    // [0, 22.5] tenths-of-degree, maps to f64 [0.0, 22.5]
    pub min_density: i32,  // >= 1
    pub max_distance: i32, // >= 0
    pub seg_size: i32,     // >= 1

    // Live edit buffers — bound to TextEdit widgets, committed on focus loss
    pub buf_angle: String,
    pub buf_density: String,
    pub buf_distance: String,
    pub buf_seg: String,
}

impl Default for ParameterSet {
    fn default() -> Self {
        Self {
            max_angle: 10,
            min_density: 2,
            max_distance: 10,
            seg_size: 5,
            // Buffers initialised to match their committed values
            buf_angle: "10".to_string(),
            buf_density: "2".to_string(),
            buf_distance: "10".to_string(),
            buf_seg: "5".to_string(),
        }
    }
}

// ─────────────────────────────────────────────
// ViewModel
// ─────────────────────────────────────────────

pub struct ViewModel {
    // File section
    pub input_file_path: String,
    pub input_name: String,
    pub num_dl: usize,
    pub percent_correlation: f64,

    // Parameters section
    pub parameter_sets: Vec<ParameterSet>,

    // Computing mode section
    pub compute_mode: ComputeMode,
}

impl Default for ViewModel {
    fn default() -> Self {
        Self {
            input_file_path: String::new(),
            input_name: String::new(),
            num_dl: 0,
            percent_correlation: 0.0,
            parameter_sets: vec![ParameterSet::default()],
            compute_mode: ComputeMode::default(),
        }
    }
}
