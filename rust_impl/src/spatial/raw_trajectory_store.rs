use crate::spatial::trajectory::Trajectory;

// TODO:
// - bucket size should be a fraction of the max angle threshold used in clustering
//   (e.g., if max angle is 5 degrees, bucket size could be 2.5 degrees to reduce sending to much neighboring buckets)
//     - Change constructor accordingly (easy)
//     - Change iter_nearby_angle accordingly (a bit more complex)

pub struct Bucket {
    pub angle_start: f64, // (inclusive)
    pub angle_end: f64,   // (exclusive)
    pub trajectories: Vec<Trajectory>,
}
pub struct RawTrajStore {
    pub bucket_size: f64,
    pub traj_buckets: Vec<Bucket>, // index-based buckets
}

impl RawTrajStore {
    pub fn new(bucket_size: f64) -> Self {
        let buckets: Vec<Bucket> = Self::create_buckets(bucket_size);

        Self {
            bucket_size,
            traj_buckets: buckets,
        }
    }

    fn create_buckets(bucket_size: f64) -> Vec<Bucket> {
        assert!(bucket_size > 0.0 && bucket_size <= 360.0);

        let num_buckets: usize = (360.0 / bucket_size).ceil() as usize;
        let mut buckets: Vec<Bucket> = Vec::with_capacity(num_buckets);

        for i in 0..num_buckets {
            let angle_start: f64 = i as f64 * bucket_size;
            let mut angle_end: f64 = angle_start + bucket_size;

            if angle_end > 360.0 {
                angle_end = 360.0;
            }

            buckets.push(Bucket {
                angle_start,
                angle_end,
                trajectories: Vec::new(),
            });
        }

        buckets
    }

    #[inline]
    fn angle_to_bucket(&self, angle: f64) -> usize {
        let mut a: f64 = angle % 360.0;
        if a < 0.0 {
            a += 360.0;
        }
        (a / self.bucket_size).floor() as usize
    }

    pub fn add_trajectory(&mut self, traj: Trajectory) {
        let bucket_idx: usize = self.angle_to_bucket(traj.angle);

        if let Some(bucket) = self.traj_buckets.get_mut(bucket_idx) {
            bucket.trajectories.push(traj);
        } else {
            panic!("Bucket index {bucket_idx} does not exist");
        }
    }

    pub fn iter_nearby_angle(&self, angle: f64) -> impl Iterator<Item = &Trajectory> {
        let idx: usize = self.angle_to_bucket(angle);
        let last: usize = self.traj_buckets.len() - 1;

        let wrap = |i: isize| -> usize {
            ((i % self.traj_buckets.len() as isize) + self.traj_buckets.len() as isize) as usize
                % self.traj_buckets.len()
        };

        let mut indices: Vec<usize> = Vec::new();

        indices.push(wrap(idx as isize - 1)); // bucket -1 (wrap-around)
        indices.push(idx); // current bucket
        indices.push(wrap(idx as isize + 1)); // bucket +1 (wrap-around)

        // --- Special +2 rule (wrap-around) ---
        let is_before_last: bool = idx == last - 1;
        let last_bucket: &Bucket = &self.traj_buckets[last];
        let last_bucket_size: f64 = last_bucket.angle_end - last_bucket.angle_start;

        if is_before_last && last_bucket_size < self.bucket_size {
            indices.push(wrap(idx as isize + 2));
        }

        indices
            .into_iter()
            .flat_map(move |i| self.traj_buckets[i].trajectories.iter())
    }

    pub fn print_summary(&self) {
        for (i, bucket) in self.traj_buckets.iter().enumerate() {
            // if bucket.trajectories.len() == 0 {
            //     continue;
            // }
            println!(
                "Bucket {}: Angle [{:.2}, {:.2}[ - {} trajectories",
                i,
                bucket.angle_start,
                bucket.angle_end,
                bucket.trajectories.len()
            );

            for traj in &bucket.trajectories {
                println!("  {}", traj.to_str());
            }
        }
    }
}
