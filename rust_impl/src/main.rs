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

// TODO: optimize with maybe the reference of the nearby_trajectories vector instead of the iterator.collect()
fn db_scan_segment_clustering(
    raw_storage: &RawTrajStore,
    args: &TraclusArgs,
) -> ClusteredTrajStore {
    let mut clust_storage: ClusteredTrajStore = ClusteredTrajStore::new(args);
    let buckets: &Vec<Bucket> = &raw_storage.traj_buckets;

    for bucket in buckets {
        let nearby_trajs: Vec<&Trajectory> =
            raw_storage.iter_nearby_angle(bucket.angle_start).collect();
        let inside_trajs: &Vec<Trajectory> = &bucket.trajectories;

        for traj_seed in inside_trajs {
            for seed in traj_seed.segments_iter() {
                let cluster_result: Option<Cluster> =
                    clust_storage.initial_segment_cluster((&seed, &traj_seed), &nearby_trajs);

                match cluster_result {
                    Some(mut cluster) => {
                        clust_storage.expand_segment_cluster(&mut cluster, &nearby_trajs);
                        cluster.print_info();
                        // clust_storage.clusters.push(cluster);
                    }
                    None => {}
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

    // raw_storage.print_summary();
    Ok(())
}
