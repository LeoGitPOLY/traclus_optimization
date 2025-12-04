use crate::cluster::cluster_member::{ClusterMember, ClusterSeed};

pub struct Cluster {
    pub seed: ClusterSeed,
    pub total_weight: u32,
    pub candidates: Vec<ClusterMember>,
    pub members: Vec<ClusterMember>,
}

impl Cluster {
    pub fn new(seed: ClusterSeed, candidates: Vec<ClusterMember>) -> Self {
        let weight: u32 = seed.cm.weight;
        Self {
            seed,
            total_weight: weight,
            candidates,
            members: Vec::new(),
        }
    }

    pub fn move_candidates_to_members(&mut self) {
        while let Some(candidate) = self.candidates.pop() {
            self.members.push(candidate);
        }
    }

    pub fn merge_clusters(&mut self, other: Cluster) {
        for candidate in other.candidates {
            if !self.contains_traj(candidate.traj_id) {
                self.total_weight += candidate.weight;
                self.candidates.push(candidate);
            }
        }
    }
    pub fn contains_traj(&self, traj_id: usize) -> bool {
        for member in &self.members {
            if member.traj_id == traj_id {
                return true;
            }
        }
        for candidate in &self.candidates {
            if candidate.traj_id == traj_id {
                return true;
            }
        }
        if self.seed.cm.traj_id == traj_id {
            return true;
        }
        false
    }

    pub fn contains_segment(&self, traj_id: usize, segment_id: usize) -> bool {
        for member in &self.members {
            if member.traj_id == traj_id && member.segment_id == segment_id {
                return true;
            }
        }
        for candidate in &self.candidates {
            if candidate.traj_id == traj_id && candidate.segment_id == segment_id {
                return true;
            }
        }
        if self.seed.cm.traj_id == traj_id && self.seed.cm.segment_id == segment_id {
            return true;
        }
        false
    }
    
    pub fn print_info(&self) {
        println!(
            "Cluster Seed ID: ({}, {}), Total Weight: {}, Num members {}, Num Candidates: {}",
            self.seed.cm.traj_id,
            self.seed.cm.segment_id,
            self.total_weight,
            self.members.len(),
            self.candidates.len(),
        );
        for member in &self.members {
            println!(
                "  Member - Traj ID: {}, Segment ID: {}, Weight: {}",
                member.traj_id, member.segment_id, member.weight
            );
        }
    }
}
