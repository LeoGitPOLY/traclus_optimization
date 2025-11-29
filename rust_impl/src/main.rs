mod spatial;
mod utils_io;

use crate::spatial::clustered_trajectory_store::Cluster;
use crate::spatial::clustered_trajectory_store::ClusterMember;
use crate::spatial::clustered_trajectory_store::ClusteredTrajStore;
use crate::spatial::geometry::Segment;
use crate::spatial::raw_trajectory_store::Bucket;
use crate::spatial::raw_trajectory_store::RawTrajStore;
use crate::spatial::trajectory::Trajectory;

use crate::utils_io::loader::parse_input_data;
use crate::utils_io::traclus_args::TraclusArgs;

use clap::Parser;
use std::io;

fn rechable_nearby_trajectory(
    seed: (&Segment, &Trajectory),
    nearby_trajectories: &Vec<&Trajectory>,
    args: &TraclusArgs,
) -> Option<Cluster> {
    let mut candidates: Vec<ClusterMember> = Vec::new();
    let mut total_weight: u32 = seed.1.weight;

    for trajectory in nearby_trajectories {
        if seed.1.id == trajectory.id {
            continue;
        }

        // angle constraint
        if (seed.1.angle - trajectory.angle).abs() > args.max_angle {
            continue;
        }
        // distance constraint
        let (dist, seg_id) = trajectory.distance_to_point(&seed.0.middle);
        if dist > args.max_dist {
            continue;
        }

        candidates.push(ClusterMember::new(trajectory.id, seg_id, &seed.0.middle));
        total_weight += trajectory.weight;
    }

    // density constraint
    if candidates.len() + 1 < args.min_density {
        return None;
    }

    let seed_member: ClusterMember = ClusterMember::new(seed.1.id, seed.0.id, &seed.0.middle);
    Some(Cluster::new(seed_member, total_weight, candidates))
}

// TODO: optimize with maybe the reference of the nearby_trajectories vector instead of the iterator.collect()
fn db_scan_segment_clustering(
    raw_storage: &RawTrajStore,
    args: &TraclusArgs,
) -> ClusteredTrajStore {
    let mut clust_storage: ClusteredTrajStore = ClusteredTrajStore::new();
    let buckets: &Vec<Bucket> = &raw_storage.traj_buckets;

    for bucket in buckets {
        let nearby_trajectories: Vec<&Trajectory> =
            raw_storage.iter_nearby_angle(bucket.angle_start).collect();
        let inside_trajectories: &Vec<Trajectory> = &bucket.trajectories;

        for traj_seed in inside_trajectories {
            for seed in traj_seed.segments_iter() {
                let cluster_result: Option<Cluster> =
                    rechable_nearby_trajectory((&seed, &traj_seed), &nearby_trajectories, &args);

                match cluster_result {
                    Some(cluster) => {
                        cluster.print_info();
                    }
                    None => {
                        // No cluster found for this seed
                    }
                }
            }
        }
    }
    clust_storage
}

fn main() -> io::Result<()> {
    let args: TraclusArgs = TraclusArgs::parse();
    let raw_storage: RawTrajStore = parse_input_data(&args);

    let clust_storage: ClusteredTrajStore = db_scan_segment_clustering(&raw_storage, &args);
    // clust_storage = clust_storage.expand_clusters(&clust_storage);

    raw_storage.print_summary();
    Ok(())
}
