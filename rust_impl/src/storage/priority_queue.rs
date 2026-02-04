// TODO: the sum of distances could be calculated only when needed, to optimize performance
// Now: it's calculated incrementally when members are added for all clusters (not for cluster in a tie)
use crate::clustering::{cluster::Cluster, cluster_member::ClusterMember};
use std::{cmp::Ordering, collections::HashSet};

pub struct PriorityQueueCluster {
    pub elements: Vec<Box<Cluster>>,
    pub non_clustered_segments: Vec<Box<ClusterMember>>,
    is_sorted: bool,
}

impl PriorityQueueCluster {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            non_clustered_segments: Vec::new(),
            is_sorted: false,
        }
    }

    pub fn push(&mut self, cluster: Cluster) {
        self.is_sorted = false;
        self.elements.push(Box::new(cluster));
    }

    fn compare_clusters(a: &Cluster, b: &Cluster) -> Ordering {
        b.total_weight
            .cmp(&a.total_weight) // descending order for weight
            .then_with(|| {
                a.sum_distance
                    .partial_cmp(&b.sum_distance) // ascending order for distance
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
        self.clean_non_clustered_segments(&used_ids);
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
        Self::remove_reversed_indexes(&mut self.elements, &remove_indexes);
    }

    #[inline]
    fn clean_individual_cluster(
        cluster: &mut Cluster,
        used: &HashSet<(usize, usize)>,
        threshold: u32,
    ) -> bool {
        // If the seed is now used, remove the entire cluster
        if used.contains(&(cluster.seed.cm.traj_id, cluster.seed.cm.segment_id)) {
            return true;
        }

        let mut remove_indexes: Vec<usize> = Vec::new();

        // Check each member is now used, remove if so
        // If total weight drops below threshold, remove entire cluster
        for (member_index, member) in cluster.members.iter().enumerate() {
            if used.contains(&(member.traj_id, member.segment_id)) {
                cluster.total_weight -= member.weight;
                remove_indexes.push(member_index);
            }

            if cluster.total_weight < threshold {
                return true;
            }
        }

        // Remove clusters members in reverse order to avoid index shifting
        Self::remove_reversed_indexes(&mut cluster.members, &remove_indexes);
        return false;
    }

    #[inline]
    fn clean_non_clustered_segments(&mut self, used: &HashSet<(usize, usize)>) {
        let mut remove_indexes: Vec<usize> = Vec::new();

        // Check each non-clustered segment is now used, remove if so
        for (index, segment) in self.non_clustered_segments.iter_mut().enumerate() {
            if used.contains(&(segment.traj_id, segment.segment_id)) {
                remove_indexes.push(index);
            }
        }

        // Remove segments in reverse order to avoid index shifting
        Self::remove_reversed_indexes(&mut self.non_clustered_segments, &remove_indexes);
    }

    #[inline]
    fn remove_reversed_indexes<T>(vec: &mut Vec<T>, indexes: &Vec<usize>) {
        for &index in indexes.iter().rev() {
            vec.remove(index);
        }
    }

    pub fn print_info(&self) {
        println!("PriorityQueueCluster info:");
        for (i, cluster) in self.elements.iter().enumerate() {
            println!(
                "Cluster {}: seed = {}, total_weight = {}, num_members = {}",
                i,
                cluster.seed.cm.traj_id,
                cluster.total_weight,
                cluster.members.len()
            );
            // print!(
            //     "    Seed: traj_id = {}, segment_id = {}, weight = {}\n",
            //     cluster.seed.cm.traj_id, cluster.seed.cm.segment_id, cluster.seed.cm.weight
            // );

            // for member in &cluster.members {
            //     println!(
            //         "    Member: traj_id = {}, segment_id = {}, starting point = ({}, {}), weight = {}",
            //         member.traj_id,
            //         member.segment_id,
            //         member.start.x,
            //         member.start.y,
            //         member.weight
            //     );
            // }
        }
    }
}
