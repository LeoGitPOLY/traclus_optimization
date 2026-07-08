use super::super::geometry::input_od_line::InputODLine;
use super::super::geometry::point::Point;
use super::super::geometry::trajectory::Trajectory;
use super::super::storage::raw_trajectories::RawTrajectories;
use crate::io::args::TraclusArgs;
use crate::utils::events::app_events::AppError;
use crate::utils::events::event_singleton::emit_error;

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

/// Detects whether the line uses tabs, commas, or semicolons as separator.
fn detect_separator(line: &str) -> char {
    if line.contains('\t') {
        '\t'
    } else if line.contains(';') {
        ';'
    } else {
        ','
    }
}

/// Parses a line into an InputODLine.
///
/// Supported formats (tab, comma, or semicolon separated):
///   With name:    name  weight  x_start  y_start  x_end  y_end
///   Without name: weight  x_start  y_start  x_end  y_end
#[inline]
fn parse_line_to_od(line: &str, index: usize) -> io::Result<InputODLine> {
    let sep: char = detect_separator(line);
    let parts: Vec<&str> = line.split(sep).map(|p| p.trim()).collect();

    let offset = match parts.len() {
        6 => 1,
        5 => 0,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Failed to parse line {}: \n\
                    Expected format is: \n\
                    \t'name, weight, x_start, y_start, x_end, y_end' or \n\
                    \t'weight, x_start, y_start, x_end, y_end'\n\
                    got:\n{}",
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

pub fn parse_input_data(args: &TraclusArgs) -> Option<RawTrajectories> {
    let content: String = match read_file(&args.file) {
        Ok(c) => c,
        Err(err) => {
            emit_error(AppError::IoError(format!(
                "Failed to read input file: {}",
                err
            )));
            return None;
        }
    };

    let mut trajectory_storage: RawTrajectories = RawTrajectories::new(args.max_angle);
    let mut number_point_lines: i32 = 0;
    for (index, line) in content.lines().enumerate() {
        let line: &str = line.trim();

        if line.is_empty() {
            continue;
        }

        if index == 0 && is_header(line) {
            continue;
        }

        let od_line: InputODLine = match parse_line_to_od(line, index + 1) {
            Ok(od) => od,
            Err(err) => {
                emit_error(AppError::IoError(format!("{}", err)));
                return None;
            }
        };

        if od_line.start == od_line.end {
            number_point_lines += 1;
            continue;
        }

        let trajectory: Trajectory = Trajectory::new(od_line, args.segment_size);
        trajectory_storage.add_trajectory(trajectory);
    }

    if number_point_lines > 0 {
        emit_error(AppError::IoError(format!(
            "WARNING: {} lines were ignored because they represent points (start and end are the same).\n\
            Consider removing these lines from the input file.",
            number_point_lines
        )));
    }

    Some(trajectory_storage)
}
