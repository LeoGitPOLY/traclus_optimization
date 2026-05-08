// Now: it's calculated incrementally when members are added for all clusters (not for cluster in a tie)

// TODO (optimization): the sum of distances could be calculated only when needed, to optimize performance

use crate::{
    io::args::{ExecutionMode, TraclusArgs},
    utils::events::event_singleton::emit_timed_perf,
};

use super::super::objects::{cluster::Cluster, cluster_member::ClusterMember};
use rayon::{iter::Enumerate, prelude::*, slice::ChunksMut};
use rustc_hash::FxHashSet;
use std::cmp::Ordering;

pub struct PriorityQueueCluster {
    pub elements: Vec<Cluster>,
    pub non_clustered_segments: Vec<ClusterMember>,
    is_initialy_sorted: bool,
}

impl PriorityQueueCluster {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            non_clustered_segments: Vec::new(),
            is_initialy_sorted: false,
        }
    }

    pub fn push(&mut self, cluster: Cluster) {
        self.is_initialy_sorted = false;
        self.elements.push(cluster);
    }

    // Ordering: first by total weight (descending), then by sum of distances (ascending)
    // First cluster to use will be at the end of the vector
    fn compare_clusters(a: &Cluster, b: &Cluster) -> Ordering {
        a.total_weight
            .cmp(&b.total_weight) // ascending order for weight
            .then_with(|| {
                b.sum_distance
                    .partial_cmp(&a.sum_distance) // descending for distance
                    .unwrap_or(Ordering::Equal)
            })
    }

    // TODO (optimization): CLAUDE.AI CHAT
    // Might not need to parallelize this, already almost in order (maybe for first iteration or when really big)
    fn sort_by_weight_and_distance(&mut self) {
        self.elements
            .sort_by(|a: &Cluster, b: &Cluster| Self::compare_clusters(a, b));
        self.is_initialy_sorted = true;
    }

    pub fn pop_and_clean(&mut self, args: &TraclusArgs) -> Option<Cluster> {
        if self.elements.is_empty() {
            return None;
        }
        if !self.is_initialy_sorted {
            emit_timed_perf("Sort_by_WeightDistance", true, None);
            self.sort_by_weight_and_distance();
            emit_timed_perf("Sort_by_WeightDistance", false, None);
        }

        emit_timed_perf("Take_First_Element", true, None);
        let last: Cluster = self.elements.pop().unwrap();
        let used_ids: FxHashSet<(usize, usize)> = Self::collect_used_traj_ids(&last);
        emit_timed_perf("Take_First_Element", false, None);

        emit_timed_perf("Clean_Remaining_Clusters", true, None);
        self.clean_remaining_clusters(&used_ids, args);
        emit_timed_perf("Clean_Remaining_Clusters", false, None);

        emit_timed_perf("Clean_NonClustered_Segments", true, None);
        self.clean_non_clustered_segments(&used_ids);
        emit_timed_perf("Clean_NonClustered_Segments", false, None);

        emit_timed_perf("Sort_by_WeightDistance", true, None);
        self.sort_by_weight_and_distance();
        emit_timed_perf("Sort_by_WeightDistance", false, None);

        Some(last)
    }

    fn collect_used_traj_ids(cluster: &Cluster) -> FxHashSet<(usize, usize)> {
        let mut set: FxHashSet<(usize, usize)> =
            FxHashSet::with_capacity_and_hasher(cluster.members.len() + 1, Default::default());

        set.insert((cluster.seed.cm.traj_id, cluster.seed.cm.segment_id));

        for member in &cluster.members {
            set.insert((member.traj_id, member.segment_id));
        }

        set
    }

    fn clean_remaining_clusters(&mut self, used: &FxHashSet<(usize, usize)>, args: &TraclusArgs) {
        const PARALLEL_THRESHOLD: usize = 10; // TODO: remove from here
        let mut mode: ExecutionMode = args.mode;

        if self.elements.len() < PARALLEL_THRESHOLD {
            mode = ExecutionMode::Serial;
        }

        let remove_indexes: Vec<usize> = match mode {
            ExecutionMode::ParallelRayon => {
                let chunk_size: usize = (self.elements.len() / rayon::current_num_threads()).max(1);
                let chunk_iter: Enumerate<ChunksMut<'_, Cluster>> =
                    self.elements.par_chunks_mut(chunk_size).enumerate();

                chunk_iter
                    .map(|(chunk_idx, chunk)| {
                        let base: usize = chunk_idx * chunk_size;
                        Self::clean_section_cluster_serial(chunk, used, args, base)
                    })
                    .flatten()
                    .collect()
            }

            ExecutionMode::Serial => {
                Self::clean_section_cluster_serial(&mut self.elements, used, args, 0)
            }
        };

        Self::remove_indexes(&mut self.elements, &remove_indexes);
    }

    #[inline]
    fn clean_section_cluster_serial(
        elements: &mut [Cluster],
        used: &FxHashSet<(usize, usize)>,
        args: &TraclusArgs,
        index_offset: usize,
    ) -> Vec<usize> {
        let thread_index = rayon::current_thread_index();
        emit_timed_perf("Clean_Clusters_serial", true, thread_index);

        let mut remove_indexes: Vec<usize> = Vec::new();

        for (index, cluster) in elements.iter_mut().enumerate() {
            if Self::clean_individual_cluster(cluster, used, args.min_density) {
                remove_indexes.push(index_offset + index);
            }
        }
        emit_timed_perf("Clean_Clusters_serial", false, thread_index);
        remove_indexes
    }

    #[inline]
    fn clean_individual_cluster(
        cluster: &mut Cluster,
        used: &FxHashSet<(usize, usize)>,
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

        Self::remove_indexes(&mut cluster.members, &remove_indexes);
        return false;
    }

    fn clean_non_clustered_segments(&mut self, used: &FxHashSet<(usize, usize)>) {
        let mut remove_indexes: Vec<usize> = Vec::new();

        // Check each non-clustered segment is now used, remove if so
        for (index, segment) in self.non_clustered_segments.iter_mut().enumerate() {
            if used.contains(&(segment.traj_id, segment.segment_id)) {
                remove_indexes.push(index);
            }
        }

        Self::remove_indexes(&mut self.non_clustered_segments, &remove_indexes);
    }

    #[inline]
    fn remove_indexes<T>(vec: &mut Vec<T>, indexes: &[usize]) {
        let to_remove: FxHashSet<usize> = indexes.iter().copied().collect();
        let mut i: usize = 0;
        vec.retain(|_| {
            let keep: bool = !to_remove.contains(&i);
            i += 1;
            keep
        });
    }

    pub fn get_size_elements(&self) -> usize {
        self.elements.len()
    }

    #[allow(unused)]
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
        }
    }
}
