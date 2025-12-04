// TODO: ADD SOME OF SQUARE AS THE TIE BREAKER WHEN WEIGHT ARE THE SAME
use std::collections::HashSet;

use crate::cluster::cluster::Cluster;

pub struct PriorityQueueCluster {
    pub elements: Vec<Box<Cluster>>,
}

impl PriorityQueueCluster {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub fn push(&mut self, cluster: Cluster) {
        self.elements.push(Box::new(cluster));
    }

    pub fn sort_by_weight(&mut self) {
        self.elements
            .sort_by(|a: &Box<Cluster>, b: &Box<Cluster>| b.total_weight.cmp(&a.total_weight));
    }


    pub fn pop_and_clean(&mut self, threshold: u32) -> Option<Box<Cluster>> {
        if self.elements.is_empty() {
            return None;
        }

        let first: Box<Cluster> = self.elements.remove(0);
        let used_ids: HashSet<usize> = Self::collect_used_traj_ids(&first);

        self.clean_remaining_clusters(&used_ids, threshold);
        self.sort_by_weight();

        Some(first)
    }

    fn collect_used_traj_ids(cluster: &Cluster) -> HashSet<usize> {
        let mut set: HashSet<usize> = HashSet::new();

        set.insert(cluster.seed.cm.traj_id);

        for m in &cluster.members {
            set.insert(m.traj_id);
        }

        set
    }

    fn clean_remaining_clusters(&mut self, used: &HashSet<usize>, threshold: u32) {
        // Use retain to keep only valid clusters
        self.elements.retain_mut(|cluster: &mut Box<Cluster>| {
            // 1) Remove used traj_id from candidates AND members
            cluster.candidates.retain(|c: &super::cluster_member::ClusterMember| !used.contains(&c.traj_id));
            cluster.members.retain(|m: &super::cluster_member::ClusterMember| !used.contains(&m.traj_id));

            // 2) If the seed is removed → drop the cluster completely
            if used.contains(&cluster.seed.cm.traj_id) {
                return false;
            }

            // 3) Recompute weight
            cluster.total_weight = recompute_cluster_weight(cluster);

            // 4) Drop if weight is too low
            cluster.total_weight >= threshold
        });
    }
}

/// Recalculate total weight from seed + members + candidates.
fn recompute_cluster_weight(c: &Cluster) -> u32 {
    let mut weight = c.seed.cm.weight;

    for m in &c.members {
        weight += m.weight;
    }
    for cand in &c.candidates {
        weight += cand.weight;
    }

    weight
}
