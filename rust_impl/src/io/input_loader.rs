use crate::geometry::input_od_line::InputODLine;
use crate::geometry::point::Point;
use crate::geometry::trajectory::Trajectory;
use crate::io::traclus_args::TraclusArgs;
use crate::storage::raw_trajectories::RawTrajectories;

use std::fs;
use std::io;
use std::path::Path;

fn read_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    fs::read_to_string(path)
}

#[inline]
fn parse_line_to_od(line: &str, index: usize) -> io::Result<InputODLine> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() != 6 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Line {} is malformed: {}", index + 1, line),
        ));
    }

    Ok(InputODLine {
        line_id: parts[0]
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to parse line_id"))?,
        weight: parts[1]
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to parse weight"))?,
        start: Point {
            x: parts[2].parse().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "Failed to parse start x")
            })?,
            y: parts[3].parse().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "Failed to parse start y")
            })?,
        },
        end: Point {
            x: parts[4]
                .parse()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to parse end x"))?,
            y: parts[5]
                .parse()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to parse end y"))?,
        },
    })
}

pub fn parse_input_data(args: &TraclusArgs) -> RawTrajectories {
    let content: String = read_file(&args.infile).expect("Failed to read input file");
    let mut trajectory_storage: RawTrajectories = RawTrajectories::new(args.max_angle);

    for (index, line) in content.lines().enumerate() {
        let od_line: InputODLine = parse_line_to_od(line, index).unwrap_or_else(|err| {
            eprintln!("{}", err);
            std::process::exit(1);
        });

        let trajectory: Trajectory = Trajectory::new(od_line, args.segment_size);
        trajectory_storage.add_trajectory(trajectory);
    }

    trajectory_storage
}
