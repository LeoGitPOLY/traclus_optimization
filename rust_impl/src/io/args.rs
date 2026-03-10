// args.rs

use clap::{Parser, ValueEnum};
use std::fmt;

use crate::io::args_config::get_param_configs;

// ─────────────────────────────────────────────
// ExecutionMode  — algorithm parallelism strategy
// ─────────────────────────────────────────────

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq)]
pub enum ExecutionMode {
    Serial,
    ParallelRayon,
}

impl fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionMode::Serial => write!(f, "Serial"),
            ExecutionMode::ParallelRayon => write!(f, "ParallelRayon"),
        }
    }
}

// ─────────────────────────────────────────────
// InterfaceMode  — which front-ends are active
// ─────────────────────────────────────────────

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum InterfaceMode {
    Gui,
    Logger,
    GuiAndLogger,
    Performance,
}

impl fmt::Display for InterfaceMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterfaceMode::Gui => write!(f, "Gui"),
            InterfaceMode::Logger => write!(f, "Logger"),
            InterfaceMode::GuiAndLogger => write!(f, "GuiAndLogger"),
            InterfaceMode::Performance => write!(f, "Performance"),
        }
    }
}

fn default_mode() -> ExecutionMode {
    ExecutionMode::Serial
}
fn default_interface_mode() -> InterfaceMode {
    InterfaceMode::Gui
}

// ─────────────────────────────────────────────
// TraclusArgs
// ─────────────────────────────────────────────

#[derive(Clone, Parser, Debug)]
#[command(author, version, about = "Traclus DL Optimized in Rust")]
pub struct TraclusArgs {
    #[arg(short = 'f', long = "file", default_value = "")]
    pub file: String,

    #[arg(
        short = 'd',
        long = "max_dist",
        default_value_t = get_param_configs().max_dist.default,
        value_parser = |v: &str| {
            let cfg = get_param_configs().max_dist;
            let val: f64 = v.parse().map_err(|_| String::from("must be a number"))?;
            if val < cfg.min || val > cfg.max {
                Err(format!("max_dist must be in range {}..={}", cfg.min, cfg.max))
            } else {
                Ok(val)
            }
        }
    )]
    pub max_dist: f64,

    #[arg(
        short = 'n',
        long = "min_density",
        default_value_t = get_param_configs().min_density.default,
        value_parser = |v: &str| {
            let cfg = get_param_configs().min_density;
            let val: u32 = v.parse().map_err(|_| String::from("must be a number"))?;
            if val < cfg.min || val > cfg.max {
                Err(format!("min_density must be in range {}..={}", cfg.min, cfg.max))
            } else {
                Ok(val)
            }
        }
    )]
    pub min_density: u32,

    #[arg(
        short = 'a',
        long = "max_angle",
        default_value_t = get_param_configs().max_angle.default,
        value_parser = |v: &str| {
            let cfg = get_param_configs().max_angle;
            let val: f64 = v.parse().map_err(|_| String::from("must be a number"))?;
            if val < cfg.min || val > cfg.max {
                Err(format!("max_angle must be in range {}..={}", cfg.min, cfg.max))
            } else {
                Ok(val)
            }
        }
    )]
    pub max_angle: f64,

    #[arg(
        short = 's',
        long = "segment_size",
        default_value_t = get_param_configs().segment_size.default,
        value_parser = |v: &str| {
            let cfg = get_param_configs().segment_size;
            let val: f64 = v.parse().map_err(|_| String::from("must be a number"))?;
            if val < cfg.min || val > cfg.max {
                Err(format!("segment_size must be in range {}..={}", cfg.min, cfg.max))
            } else {
                Ok(val)
            }
        }
    )]
    pub segment_size: f64,

    #[arg(short = 'm', long = "mode",      value_enum, default_value_t = default_mode())]
    pub mode: ExecutionMode,

    #[arg(short = 'i', long = "interface", value_enum, default_value_t = default_interface_mode())]
    pub interface_mode: InterfaceMode,
}

impl Default for TraclusArgs {
    fn default() -> Self {
        let cfg = get_param_configs();
        Self {
            file: String::new(),
            max_dist: cfg.max_dist.default,
            min_density: cfg.min_density.default,
            max_angle: cfg.max_angle.default,
            segment_size: cfg.segment_size.default,
            mode: default_mode(),
            interface_mode: default_interface_mode(),
        }
    }
}
