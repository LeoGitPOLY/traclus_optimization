use crate::spatial::geometry::{Point, Segment};
use crate::spatial::trajectory::Trajectory;
use crate::utils_io::traclus_args::TraclusArgs;
use crate::cluster::cluster_member::{ClusterMember, ClusterSeed};
use crate::cluster::cluster::Cluster;

pub struct ClusteredTrajStore {
    args: TraclusArgs,
    // clusters: Vec<Box<Cluster>>,
}

impl ClusteredTrajStore {
    pub fn new(args: &TraclusArgs) -> Self {
        Self { args: args.clone() }
    }

    /* Méthode pour agréger un cluster autour d'un segment seed
    Vérifie les contraintes: 1) Pas la même trajectoire, 2) Angle, 3) Distance 4) Densité
    Time complexity: O(n x d / bucket_size)
    */
    fn cluster_reachable_segs(
        &self,
        seed: ClusterSeed,
        nearby_trajs: &Vec<&Trajectory>,
    ) -> Option<Cluster> {
        let mut cluster: Cluster = Cluster::new(seed, Vec::new());
        let seed_ref: &ClusterSeed = &cluster.seed;

        for nearby_traj in nearby_trajs {
            // SAME TRAJECTORY CONSTRAINT
            if seed_ref.cm.traj_id == nearby_traj.id {
                continue;
            }

            // ANGLE CONSTRAINT
            if (seed_ref.angle - nearby_traj.angle).abs() > self.args.max_angle {
                continue;
            }

            // DISTANCE CONSTRAINT
            let (dist, segment_id) = nearby_traj.distance_to_point(&seed_ref.cm.center_point);
            if dist > self.args.max_dist {
                continue;
            }

            let center_segment: Point = nearby_traj.segment(segment_id).unwrap().middle.clone();
            let candidate: ClusterMember = ClusterMember::new(
                nearby_traj.id,
                segment_id,
                nearby_traj.weight,
                center_segment,
            );
            cluster.total_weight += candidate.weight;
            cluster.candidates.push(candidate);
        }

        // DENSITY CONSTRAINT (including the seed)
        if cluster.total_weight < self.args.min_density {
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
            cluster.move_candidates_to_members();
            let mut new_clusters: Vec<Cluster> = Vec::new();

            // utiliser chaque membre du cluster comme un nouveau seed pour trouver des nouveaux segments atteignables
            for members in &cluster.members {
                let seed_member: ClusterSeed = ClusterSeed::new(
                    ClusterMember::new(
                        members.traj_id,
                        members.segment_id,
                        members.weight,
                        members.center_point.clone(),
                    ),
                    cluster.seed.angle,
                );
                if let Some(new_cluster) = self.cluster_reachable_segs(seed_member, nearby_trajs) {
                    new_clusters.push(new_cluster);
                } else {
                    continue;
                }
            }

            // fusionner les nouveaux clusters dans le cluster de base
            for new_cluster in new_clusters {
                cluster.merge_clusters(new_cluster);
            }
        }
        cluster
    }

    /* Méthode initier le clustering à partir d'un segment seed
    Time complexity: O(cluster_reachable_segs) = O(n x d / bucket_size)
    */
    pub fn initial_segment_cluster(
        &self,
        seed: (&Segment, &Trajectory),
        nearby_trajs: &Vec<&Trajectory>,
    ) -> Option<Cluster> {
        let seed_member: ClusterSeed = ClusterSeed::new(
            ClusterMember::new(seed.1.id, seed.0.id, seed.1.weight, seed.0.middle.clone()),
            seed.1.angle,
        );
        self.cluster_reachable_segs(seed_member, nearby_trajs)
    }
}
