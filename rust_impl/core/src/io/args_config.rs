// args_config.rs - Single source of truth for all args constraints and defaults.
// Change values here only — TraclusArgs and the GUI both read from these structs.

// ─────────────────────────────────────────────
// ArgsConfig — constraints for one argument
// ─────────────────────────────────────────────

use crate::utils::angle_u16::AngleU16;

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
    pub max_threads: ArgsConfig<u32>,
    pub num_fields_map: ArgsConfig<usize>,
}

/// Call this from any module that needs defaults, min, or max.
pub fn get_param_configs() -> AllArgsConfigs {
    AllArgsConfigs {
        segment_size: ArgsConfig {
            default: 500.0,
            min: f64::MIN_POSITIVE, // > 0
            max: f64::MAX,
            label: "SEG SIZE",
        },
        max_angle: ArgsConfig {
            default: 5.0,
            min: AngleU16::MIN_POSITIVE.to_degrees(), // > 0.01
            max: 22.5,
            label: "MAX ANGLE",
        },
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
        max_threads: ArgsConfig {
            default: u32::MAX,
            min: 0,
            max: u32::MAX,
            label: "MAX THREADS",
        },
        num_fields_map: ArgsConfig {
            default: 5,
            min: 5,
            max: 6,
            label: "NUM FIELDS MAP",
        },
    }
}
