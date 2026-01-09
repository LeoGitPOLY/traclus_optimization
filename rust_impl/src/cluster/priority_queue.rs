// TODO: the sum of distances could be calculated only when needed, to optimize performance
// Now: it's calculated incrementally when members are added for all clusters (not for cluster in a tie)
use crate::cluster::cluster::Cluster;
use std::{cmp::Ordering, collections::HashSet};

pub struct PriorityQueueCluster {
    pub elements: Vec<Box<Cluster>>,
    is_sorted: bool,
}

impl PriorityQueueCluster {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            is_sorted: false,
        }
    }

    pub fn push(&mut self, cluster: Cluster) {
        self.is_sorted = false;
        self.elements.push(Box::new(cluster));
    }

    fn compare_clusters(a: &Cluster, b: &Cluster) -> Ordering {
        b.total_weight
            .cmp(&a.total_weight) // descending weight
            .then_with(|| {
                a.sum_distance
                    .partial_cmp(&b.sum_distance) // ascending distance
                    .unwrap_or(Ordering::Equal)
            })
    }

    fn sort_by_weight_and_distance(&mut self) {
        self.elements
            .sort_by(|a: &Box<Cluster>, b: &Box<Cluster>| Self::compare_clusters(a, b));
        self.is_sorted = true;
    }

    pub fn pop_and_clean(&mut self, threshold: u32) -> Option<Box<Cluster>> {
        if self.elements.is_empty() {
            return None;
        }
        if !self.is_sorted {
            self.sort_by_weight_and_distance();
        }

        let first: Box<Cluster> = self.elements.remove(0); // TODO: optimize later with something not O(n)
        let used_ids: HashSet<(usize, usize)> = Self::collect_used_traj_ids(&first);

        self.clean_remaining_clusters(&used_ids, threshold);
        self.sort_by_weight_and_distance();

        Some(first)
    }

    fn collect_used_traj_ids(cluster: &Cluster) -> HashSet<(usize, usize)> {
        let mut set: HashSet<(usize, usize)> = HashSet::new();

        set.insert((cluster.seed.cm.traj_id, cluster.seed.cm.segment_id));

        for member in &cluster.members {
            set.insert((member.traj_id, member.segment_id));
        }

        set
    }

    fn clean_remaining_clusters(&mut self, used: &HashSet<(usize, usize)>, threshold: u32) {
        let mut remove_indexes: Vec<usize> = Vec::new();

        for (index, cluster) in self.elements.iter_mut().enumerate() {
            if Self::clean_individual_cluster(cluster, used, threshold) {
                remove_indexes.push(index);
            }
        }

        // Remove clusters in reverse order to avoid index shifting
        for &index in remove_indexes.iter().rev() {
            self.elements.remove(index);
        }
    }

    #[inline]
    fn clean_individual_cluster(
        cluster: &mut Cluster,
        used: &HashSet<(usize, usize)>,
        threshold: u32,
    ) -> bool {
        if used.contains(&(cluster.seed.cm.traj_id, cluster.seed.cm.segment_id)) {
            return true;
        }

        let mut remove_indexes: Vec<usize> = Vec::new();

        for (member_index, member) in cluster.members.iter().enumerate() {
            if used.contains(&(member.traj_id, member.segment_id)) {
                cluster.total_weight -= member.weight;
                remove_indexes.push(member_index);
            }

            if cluster.total_weight < threshold {
                return true;
            }
        }

        // Remove clusters in reverse order to avoid index shifting
        for &index in remove_indexes.iter().rev() {
            cluster.members.remove(index);
        }
        return false;
    }
}
