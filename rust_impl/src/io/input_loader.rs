use crate::clustering::geometry::input_od_line::InputODLine;
use crate::clustering::geometry::point::Point;
use crate::clustering::geometry::trajectory::Trajectory;
use crate::clustering::storage::raw_trajectories::RawTrajectories;
use crate::gui::app_events::AppError;
use crate::gui::app_events::ComputationEvent;
use crate::io::args::TraclusArgs;

use std::fs;
use std::io;
use std::path::Path;

fn read_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    fs::read_to_string(path)
}

/// Detects if a line is a header by checking if any field is non-numeric.
fn is_header(line: &str) -> bool {
    let sep = detect_separator(line);
    line.split(sep)
        .any(|field| field.trim().parse::<f64>().is_err())
}

/// Detects whether the line uses tabs or commas as separator.
fn detect_separator(line: &str) -> char {
    if line.contains('\t') { '\t' } else { ',' }
}

/// Parses a line into an InputODLine.
///
/// Supported formats (tab or comma separated):
///   With name:    name  weight  x_start  y_start  x_end  y_end
///   Without name: weight  x_start  y_start  x_end  y_end
#[inline]
fn parse_line_to_od(line: &str, index: usize) -> io::Result<InputODLine> {
    let sep = detect_separator(line);
    let parts: Vec<&str> = line.split(sep).map(|p| p.trim()).collect();

    let offset = match parts.len() {
        6 => 1,
        5 => 0,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Failed to parse line {}: expected format is \
                     'name weight x_start y_start x_end y_end' or \
                     'weight x_start y_start x_end y_end', got: {}",
                    index, line
                ),
            ));
        }
    };

    let weight: u32 = parts[offset].parse::<u32>().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Failed to parse weight at line {}: '{}'",
                index, parts[offset]
            ),
        )
    })?;

    let x_start: f64 = parts[offset + 1].parse::<f64>().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Failed to parse x_start at line {}: '{}'",
                index,
                parts[offset + 1]
            ),
        )
    })?;

    let y_start: f64 = parts[offset + 2].parse::<f64>().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Failed to parse y_start at line {}: '{}'",
                index,
                parts[offset + 2]
            ),
        )
    })?;

    let x_end: f64 = parts[offset + 3].parse::<f64>().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Failed to parse x_end at line {}: '{}'",
                index,
                parts[offset + 3]
            ),
        )
    })?;

    let y_end: f64 = parts[offset + 4].parse::<f64>().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Failed to parse y_end at line {}: '{}'",
                index,
                parts[offset + 4]
            ),
        )
    })?;

    Ok(InputODLine {
        line_id: index,
        weight,
        start: Point {
            x: x_start,
            y: y_start,
        },
        end: Point { x: x_end, y: y_end },
    })
}

pub fn parse_input_data(
    args: &TraclusArgs,
    emitter: &mut ComputationEvent,
) -> Option<RawTrajectories> {
    let content = match read_file(&args.file) {
        Ok(c) => c,
        Err(err) => {
            emitter.emit_error(AppError::IoError(format!(
                "Failed to read input file: {}",
                err
            )));
            return None;
        }
    };

    let mut trajectory_storage = RawTrajectories::new(args.max_angle);
    for (index, line) in content.lines().enumerate() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        if index == 0 && is_header(line) {
            continue;
        }

        let od_line = match parse_line_to_od(line, index + 1) {
            Ok(od) => od,
            Err(err) => {
                emitter.emit_error(AppError::IoError(format!("{}", err)));
                return None;
            }
        };

        let trajectory: Trajectory = Trajectory::new(od_line, args.segment_size);
        trajectory_storage.add_trajectory(trajectory);
    }

    Some(trajectory_storage)
}
