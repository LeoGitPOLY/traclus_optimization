// args.rs

use clap::{Parser, ValueEnum};
use std::fmt;

// ─────────────────────────────────────────────
// ExecutionMode enum : algorithm parallelism strategy
// ─────────────────────────────────────────────
#[derive(Copy, Clone, Debug, ValueEnum)]
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
// InterfaceMode enum : which front-ends are active
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

// ─────────────────────────────────────────────
// Default value functions
// ─────────────────────────────────────────────

fn default_max_dist() -> f64 {
    250.0
}
fn default_min_density() -> u32 {
    3
}
fn default_max_angle() -> f64 {
    5.0
}
fn default_segment_size() -> f64 {
    500.0
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
    #[arg(short = 'f', long)]
    pub file: String,

    #[arg(
        short = 'd',
        long = "max_dist",
        default_value_t = default_max_dist(),
        value_parser = |v: &str| {
            let val: f64 = v.parse().map_err(|_| String::from("must be a number"))?;
            if val < 0.0 {
                Err(String::from("max_dist must be >= 0"))
            } else {
                Ok(val)
            }
        }
    )]
    pub max_dist: f64,

    #[arg(
        short = 'n',
        long = "min_density",
        default_value_t = default_min_density(),
        value_parser = |v: &str| {
            let val: u32 = v.parse().map_err(|_| String::from("must be a number"))?;
            if val < 1 {
                Err(String::from("min_density must be >= 1"))
            } else {
                Ok(val)
            }
        }
    )]
    pub min_density: u32,

    #[arg(
        short = 'a',
        long = "max_angle",
        default_value_t = default_max_angle(),
        value_parser = |v: &str| {
            let val: f64 = v.parse().map_err(|_| String::from("must be a number"))?;
            if val < 0.0 || val > 22.5 {
                Err(String::from("max_angle must be in range 0.0..22.5"))
            } else {
                Ok(val)
            }
        }
    )]
    pub max_angle: f64,

    #[arg(
        short = 's',
        long = "segment_size",
        default_value_t = default_segment_size(),
        value_parser = |v: &str| {
            let val: f64 = v.parse().map_err(|_| String::from("must be a number"))?;
            if val <= 0.0 {
                Err(String::from("segment_size must be > 0"))
            } else {
                Ok(val)
            }
        }
    )]
    pub segment_size: f64,

    #[arg(
        short = 'm',
        long = "mode",
        value_enum,
        default_value_t = default_mode()
    )]
    pub mode: ExecutionMode,

    #[arg(
        short = 'i',
        long = "interface",
        value_enum,
        default_value_t = default_interface_mode()
    )]
    pub interface_mode: InterfaceMode,
}

impl Default for TraclusArgs {
    fn default() -> Self {
        Self {
            file: String::new(),
            max_dist: default_max_dist(),
            min_density: default_min_density(),
            max_angle: default_max_angle(),
            segment_size: default_segment_size(),
            mode: default_mode(),
            interface_mode: default_interface_mode(),
        }
    }
}
