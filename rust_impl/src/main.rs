mod spatial;
mod utils_io;

use crate::spatial::raw_trajectory_store::Bucket;
use crate::spatial::raw_trajectory_store::RawTrajectoryStore;
use crate::spatial::trajectory::Trajectory;

use crate::utils_io::loader::parse_input_data;
use crate::utils_io::traclus_args::TraclusArgs;

use clap::Parser;
use std::io;

// TODO: optimize with maybe the reference of the nearby_trajectories vector instead of the iterator.collect()
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
    let raw_storage: RawTrajectoryStore = parse_input_data(&args);

    db_scan_segment_clustering(&raw_storage, &args);
    raw_storage.print_summary();
    Ok(())
}
