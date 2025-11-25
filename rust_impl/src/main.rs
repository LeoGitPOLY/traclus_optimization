mod spatial;
mod utils_io;

use crate::spatial::geometry::Point;
use crate::spatial::input_od_line::InputODLine;
use crate::spatial::raw_trajectory_store::Bucket;
use crate::spatial::raw_trajectory_store::RawTrajectoryStore;
use crate::spatial::trajectory::Trajectory;
use crate::utils_io::traclus_args::TraclusArgs;

use clap::Parser;
use std::fs;
use std::io;
use std::path::Path;

fn read_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    fs::read_to_string(path)
}

fn parse_od_lines(args: &TraclusArgs) -> RawTrajectoryStore {
    let content: String = read_file(&args.infile).expect("Failed to read input file");
    let mut trajectory_storage: RawTrajectoryStore = RawTrajectoryStore::new(args.max_angle);

    for (index, line) in content.lines().enumerate() {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() != 6 {
            eprintln!("Warning: line {} is malformed: {}", index + 1, line);
            continue;
        }

        let od_line: InputODLine = InputODLine {
            line_id: parts[0].parse().unwrap(),
            weight: parts[1].parse().unwrap(),
            start: Point {
                x: parts[2].parse().unwrap(),
                y: parts[3].parse().unwrap(),
            },
            end: Point {
                x: parts[4].parse().unwrap(),
                y: parts[5].parse().unwrap(),
            },
        };
        let trajectory: Trajectory = Trajectory::new(od_line);
        trajectory_storage.add_trajectory(trajectory);
    }

    trajectory_storage
}

// TODO: optimize with maybe the reference of the nearby_trajectories instead of the iterator.collect()
fn db_scan_segment_clustering(raw_storage: &RawTrajectoryStore, args: &TraclusArgs) {
    let buckets: &Vec<Bucket> = &raw_storage.traj_buckets;

    for bucket in buckets {
        let bucket_angle: f64 = bucket.angle_start;
        let nearby_trajectories: Vec<&Trajectory> =
            raw_storage.iter_nearby_angle(bucket_angle).collect();
        let inside_segments: &Vec<Trajectory> = &bucket.trajectories;

        for segment in inside_segments {
            for trajectory in &nearby_trajectories {
                if (segment.angle - trajectory.angle).abs() > args.max_angle {
                    continue;
                }

                if segment.id == trajectory.id {
                    continue;
                }

                print!(
                    "Segment {} and Trajectory {} are nearby\n",
                    segment.id, trajectory.id
                );
            }
        }
    }
}
fn main() -> io::Result<()> {
    let args: TraclusArgs = TraclusArgs::parse();
    let raw_storage: RawTrajectoryStore = parse_od_lines(&args);

    db_scan_segment_clustering(&raw_storage, &args);
    raw_storage.print_summary();
    Ok(())
}
