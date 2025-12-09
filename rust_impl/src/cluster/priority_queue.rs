// TODO: ADD SOME OF SQUARE AS THE TIE BREAKER WHEN WEIGHT ARE THE SAME
use std::{collections::HashSet, mem};

use crate::cluster::cluster::Cluster;

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

    pub fn sort_by_weight(&mut self) {
        self.elements
            .sort_by(|a: &Box<Cluster>, b: &Box<Cluster>| b.total_weight.cmp(&a.total_weight));
        self.is_sorted = true;
    }

    pub fn pop_and_clean(&mut self, threshold: u32) -> Option<Box<Cluster>> {
        if self.elements.is_empty() {
            return None;
        }
        if !self.is_sorted {
            self.sort_by_weight();
        }

        let first: Box<Cluster> = self.elements.remove(0); // TODO: optimize later with something not O(n)
        let used_ids: HashSet<(usize, usize)> = Self::collect_used_traj_ids(&first);

        self.clean_remaining_clusters(&used_ids, threshold);
        self.sort_by_weight();

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
            if used.contains(&(cluster.seed.cm.traj_id, cluster.seed.cm.segment_id)) {
                remove_indexes.push(index);
                continue;
            }

            for member in &cluster.members {
                if used.contains(&(member.traj_id, member.segment_id)) {
                    cluster.total_weight -= member.weight;
                }
            }

            if cluster.total_weight < threshold {
                remove_indexes.push(index);
                continue;
            }
        }

        // Remove clusters in reverse order to avoid index shifting
        for &index in remove_indexes.iter().rev() {
            self.elements.remove(index);
        }
    }
}