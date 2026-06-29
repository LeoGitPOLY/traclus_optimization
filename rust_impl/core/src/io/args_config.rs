// args_config.rs - Single source of truth for all args constraints and defaults.
// Change values here only — TraclusArgs and the GUI both read from these structs.

// ─────────────────────────────────────────────
// ArgsConfig — constraints for one argument
// ─────────────────────────────────────────────

pub struct ArgsConfig<T> {
    pub default: T,
    pub min: T,
    pub max: T,
    pub label: &'static str, // display name used in GUI headers and CLI help
}

// ─────────────────────────────────────────────
// AllArgsConfigs — the full set, returned as one struct
// ─────────────────────────────────────────────

pub struct AllArgsConfigs {
    pub max_dist: ArgsConfig<f64>,
    pub min_density: ArgsConfig<u32>,
    pub max_angle: ArgsConfig<f64>,
    pub segment_size: ArgsConfig<f64>,
}

/// Call this from any module that needs defaults, min, or max.
pub fn get_param_configs() -> AllArgsConfigs {
    AllArgsConfigs {
        max_dist: ArgsConfig {
            default: 250.0,
            min: 0.0,
            max: f64::MAX,
            label: "MAX DISTANCE",
        },
        min_density: ArgsConfig {
            default: 3,
            min: 1,
            max: u32::MAX,
            label: "MIN DENSITY",
        },
        max_angle: ArgsConfig {
            default: 5.0,
            min: f64::MIN_POSITIVE, // > 0
            max: 22.5,
            label: "MAX ANGLE",
        },
        segment_size: ArgsConfig {
            default: 500.0,
            min: f64::MIN_POSITIVE, // > 0
            max: f64::MAX,
            label: "SEG SIZE",
        },
    }
}
