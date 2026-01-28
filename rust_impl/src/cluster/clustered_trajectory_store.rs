use crate::cluster::cluster::Cluster;
use crate::cluster::cluster_member::{ClusterMember, ClusterSeed};
use crate::cluster::priority_queue::PriorityQueueCluster;
use crate::spatial::geometry::{Corridor, Point, Segment};
use crate::spatial::trajectory::Trajectory;
use crate::utils_io::traclus_args::TraclusArgs;

pub struct ClusteredTrajStore {
    args: TraclusArgs,
    pub clusters: PriorityQueueCluster,
    pub corridors: Vec<Corridor>,
}

impl ClusteredTrajStore {
    pub fn new(args: &TraclusArgs) -> Self {
        Self {
            args: args.clone(),
            clusters: PriorityQueueCluster::new(),
            corridors: Vec::new(),
        }
    }

    /* Méthode pour agréger un cluster autour d'un segment seed
    Vérifie les contraintes: 1) Pas la même trajectoire, 2) Angle, 3) Distance 4) Densité
    Time complexity: O(n x d / bucket_size)
    */
    fn cluster_reachable_segs(
        &self,
        seed: ClusterSeed,
        nearby_trajs: &Vec<&Trajectory>,
        id_expended: usize,
    ) -> Option<Cluster> {
        let mut cluster: Cluster = Cluster::new(seed, Vec::new());
        let seed_ref: &ClusterSeed = &cluster.seed;
        let mut local_weight: u32 = seed_ref.cm.weight;

        for nearby_traj in nearby_trajs {
            // SAME TRAJECTORY CONSTRAINT
            if seed_ref.cm.traj_id == nearby_traj.id {
                continue;
            }

            // ANGLE CONSTRAINT
            let angle_diff: f64 = (seed_ref.angle - nearby_traj.angle).abs();
            let min_angle_diff: f64 = angle_diff.min(360.0 - angle_diff);
            if min_angle_diff > self.args.max_angle + 1e-9 {
                continue;
            }

            // DISTANCE CONSTRAINT
            let (dist, segment_id) = nearby_traj.distance_to_point(&seed_ref.cm.center);
            if dist > self.args.max_dist + 1e-9 {
                continue;
            }

            let segment: &Segment = nearby_traj.segment(segment_id).unwrap();
            let center_segment: Point = segment.middle.clone();
            let start_segment: Point = segment.start.clone();
            let candidate: ClusterMember = ClusterMember::new(
                nearby_traj.id,
                segment_id,
                nearby_traj.weight,
                center_segment,
                start_segment,
            );
            local_weight += candidate.weight;
            cluster.candidates.push(candidate);
        }

        // TODO remove:
        let list_traj_debug = vec![
            701, 1017, 970, 969, 2949, 2282, 2444, 422, 2298, 1810, 298, 1009, 516, 2562, 2045,
            667, 379, 2478, 404, 2999, 230, 2957, 380, 938, 2073, 1994, 496, 2208, 2620, 2438,
            2752,
        ];

        if cluster
            .candidates
            .iter()
            .any(|c: &ClusterMember| c.traj_id == 2728)
            && list_traj_debug.contains(&seed_ref.cm.traj_id)
            && id_expended == 701
        {
            println!(
                "$$$$       Checking traj 2728 to seed traj {}, local_weight = {}",
                seed_ref.cm.traj_id, local_weight
            );
            for candidate in &cluster.candidates {
                println!(
                    "    Candidate traj_id = {}, segment_id = {}, weight = {}",
                    candidate.traj_id, candidate.segment_id, candidate.weight
                );
            }
        }

        // DENSITY CONSTRAINT (including the seed)
        if local_weight < self.args.min_density {
            return None;
        }
        Some(cluster)
    }

    /* Méthode pour étendre le cluster en agrégeant à partir des segments candidats
    Time complexity: O(m' x cluster_reachable_segs) = O(m' x n x d / bucket_size)
    */
    pub fn expand_segment_cluster<'a>(
        &self,
        cluster: &'a mut Cluster,
        nearby_trajs: &Vec<&Trajectory>,
    ) -> &'a mut Cluster {
        while cluster.candidates.len() > 0 {
            let mut new_clusters: Vec<Cluster> = Vec::new();

            // Iterate in REVERSE order to match v1's pop() behavior
            for candidate in cluster.candidates.iter().rev() {
                let seed_member: ClusterSeed = ClusterSeed::new(
                    ClusterMember::new_from_candidate(candidate),
                    cluster.seed.angle,
                );
                if let Some(new_cluster) =
                    self.cluster_reachable_segs(seed_member, nearby_trajs, cluster.seed.cm.traj_id)
                {
                    new_clusters.push(new_cluster);
                }
            }

            cluster.move_candidates_to_members();

            // fusionner les nouveaux clusters dans le cluster de base
            for new_cluster in new_clusters {
                cluster.merge_clusters(new_cluster);
            }
        }
        cluster
    }

    /* Méthode pour initier le clustering à partir d'un segment seed
    Time complexity: O(cluster_reachable_segs) = O(n x d / bucket_size)
    */
    pub fn initial_segment_cluster(
        &self,
        seed: (&Segment, &Trajectory),
        nearby_trajs: &Vec<&Trajectory>,
    ) -> Option<Cluster> {
        let member: ClusterMember = ClusterMember::new_from_traj(seed.1, seed.0);
        let seed_member: ClusterSeed = ClusterSeed::new(member, seed.1.angle);
        self.cluster_reachable_segs(seed_member, nearby_trajs, 0)
    }

    pub fn pop_completed_cluster(&mut self) -> Option<Box<Cluster>> {
        self.clusters.pop_and_clean(self.args.min_density)
    }
}
