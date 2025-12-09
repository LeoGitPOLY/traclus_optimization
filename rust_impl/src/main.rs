mod cluster;
mod spatial;
mod utils_io;

use crate::cluster::cluster::Cluster;
use crate::cluster::clustered_trajectory_store::ClusteredTrajStore;
use crate::spatial::raw_trajectory_store::Bucket;
use crate::spatial::raw_trajectory_store::RawTrajStore;
use crate::spatial::trajectory::Trajectory;

use crate::utils_io::loader::parse_input_data;
use crate::utils_io::traclus_args::TraclusArgs;

use clap::Parser;
use std::io;

fn create_corridors(raw_storage: &RawTrajStore, clust_storage: &mut ClusteredTrajStore) {
    while let Some(complet_clust) = clust_storage.pop_complete_cluster() {
        complet_clust.print_info();
    }
}

// TODO: optimize with maybe the reference of the nearby_trajectories vector instead of the iterator.collect()
/// Performs a version of DBSCAN clustering on trajectory segments organized in angle-based buckets.
///
/// This function:
/// 1. Iterates through all trajectory segments in angle-ordered buckets
/// 2. Attempts to form initial clusters from seed segments
/// 3. Expands valid clusters by finding nearby dense regions
/// 4. Returns a collection of all discovered clusters
fn db_scan_segment_clustering(
    raw_storage: &RawTrajStore,
    args: &TraclusArgs,
) -> ClusteredTrajStore {
    let mut clust_storage: ClusteredTrajStore = ClusteredTrajStore::new(args);

    // Process each angle bucket and its trajectories
    for bucket in &raw_storage.traj_buckets {
        // Get nearby trajectories for this angle bucket: contains all trajectories within angle range
        let nearby_trajs: Vec<&Trajectory> =
            raw_storage.iter_nearby_angle(bucket.angle_start).collect();

        // Process each trajectory and its segments within this bucket
        for traj_seed in &bucket.trajectories {
            process_trajectory_segments(traj_seed, &nearby_trajs, &mut clust_storage);
        }
    }

    clust_storage
}

/// Processes all segments in a trajectory, attempting to form and expand clusters.
///
/// For each segment:
/// - Attempts to create an initial cluster if density requirements are met
/// - Expands the cluster to include all reachable segments
/// - Stores the completed cluster
#[inline]
fn process_trajectory_segments(
    traj_seed: &Trajectory,
    nearby_trajs: &Vec<&Trajectory>,
    clust_storage: &mut ClusteredTrajStore,
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

fn main() -> io::Result<()> {
    let args: TraclusArgs = TraclusArgs::parse();
    let raw_storage: RawTrajStore = parse_input_data(&args);

    let mut clust_storage: ClusteredTrajStore = db_scan_segment_clustering(&raw_storage, &args);
    create_corridors(&raw_storage, &mut clust_storage);

    Ok(())
}
