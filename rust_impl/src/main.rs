mod clustering;
mod geometry;
mod io;
mod storage;

use crate::clustering::corridor::Corridor;
use crate::geometry::trajectory::Trajectory;
use crate::storage::clustered_trajectories::ClusteredTrajectories;
use crate::storage::raw_trajectories::RawTrajectories;

use crate::io::loader::{parse_input_data, parse_output_data};
use crate::io::traclus_args::TraclusArgs;

use clap::Parser;

fn create_corridors(clust_storage: &mut ClusteredTrajectories) {
    // clust_storage.clusters.sort_by_weight_and_distance();
    // clust_storage.clusters.print_info();

    while let Some(completed_cluster) = clust_storage.pop_completed_cluster() {
        let index_corridor: usize = clust_storage.corridors.len();
        clust_storage
            .corridors
            .push(Corridor::new(&completed_cluster, index_corridor));
    }
}

// TODO: optimize with maybe the reference of the nearby_trajectories vector instead of the iterator.collect()
/// Performs a version of DBSCAN clustering on trajectory segments organized in angle-based buckets.
///
/// This function:
/// 1. Iterates through all trajectory segments in angle-ordered buckets
/// 2. Attempts to form initial clusters from seed segments
/// 3. Expands valid clusters by finding nearby dense regions
/// 4. Set a collection of all discovered clusters inside the clustered trajectory storage
fn db_scan_clustering(raw_storage: &RawTrajectories, clust_storage: &mut ClusteredTrajectories) {
    // Process each angle bucket and its trajectories
    for bucket in &raw_storage.traj_buckets {
        // Get nearby trajectories for this angle bucket: contains all trajectories within angle range
        let nearby_trajs: Vec<&Trajectory> =
            raw_storage.iter_nearby_angle(bucket.angle_start).collect();

        for traj_seed in &bucket.trajectories {
            trajectory_segments_clustering(traj_seed, &nearby_trajs, clust_storage);
        }
    }
}

/// Clusters all segments in a trajectory attempting to form and expand clusters.
///
/// For each segment:
/// - Attempts to create an initial cluster if density requirements are met
/// - Expands the cluster to include all reachable segments
/// - Stores the completed cluster
#[inline]
fn trajectory_segments_clustering(
    traj_seed: &Trajectory,
    nearby_trajs: &Vec<&Trajectory>,
    clust_storage: &mut ClusteredTrajectories,
) {
    for seed_segment in traj_seed.segments_iter() {
        // Try to form an initial cluster from this seed segment
        if let Some(mut cluster) =
            clust_storage.initial_segment_cluster((&seed_segment, &traj_seed), nearby_trajs)
        {
            // Expand the cluster to include all density-reachable segments
            clust_storage.expand_segment_cluster(&mut cluster, nearby_trajs);
            clust_storage.clusters.push(cluster);
        }
        // If no cluster forms, continue to next segment (not dense enough)
    }
}

fn main() -> std::io::Result<()> {
    let args: TraclusArgs = TraclusArgs::parse();
    let raw_storage: RawTrajectories = parse_input_data(&args);
    let mut clust_storage: ClusteredTrajectories = ClusteredTrajectories::new(&args);

    db_scan_clustering(&raw_storage, &mut clust_storage);
    create_corridors(&mut clust_storage);
    parse_output_data(&args, &clust_storage);

    Ok(())
}
