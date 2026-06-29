use super::super::geometry::trajectory::Trajectory;

const BUCKET_SIZE: f64 = 1.0; // degrees, must evenly divide 360.0

pub struct Bucket {
    pub angle_start: f64, // (inclusive)
    pub angle_end: f64,   // (exclusive)
    pub trajectories: Vec<Trajectory>,
}
pub struct RawTrajectories {
    pub bucket_size: f64,
    pub max_angle: f64,
    pub traj_buckets: Vec<Bucket>,
}

impl RawTrajectories {
    pub fn new(max_angle: f64) -> Self {
        let buckets: Vec<Bucket> = Self::create_buckets(BUCKET_SIZE);

        Self {
            bucket_size: BUCKET_SIZE,
            max_angle,
            traj_buckets: buckets,
        }
    }

    fn create_buckets(bucket_size: f64) -> Vec<Bucket> {
        let num_buckets: usize = (360.0 / bucket_size).round() as usize;
        assert!(bucket_size > 0.0 && bucket_size <= 360.0);
        assert!(
            (num_buckets as f64 * bucket_size - 360.0).abs() < 1e-9,
            "Bucket size must evenly divide 360"
        );

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

    // Returns an iterator over trajectories from all buckets within max_angle
    pub fn iter_nearby_angle(&self, angle: f64) -> impl Iterator<Item = &Trajectory> {
        let idx: usize = self.angle_to_bucket(angle);

        let u_len: usize = self.traj_buckets.len();
        let i_len: isize = u_len as isize;

        let wrap = |i: isize| -> usize { ((i % i_len) + i_len) as usize % u_len };

        // Number of neighboring buckets required on each side to fully cover max_angle
        let bucket_radius: isize = (self.max_angle / self.bucket_size).ceil() as isize;
        let mut indices: Vec<usize> = Vec::new();

        for offset in -bucket_radius..=bucket_radius {
            indices.push(wrap(idx as isize + offset));
        }

        indices
            .into_iter()
            .flat_map(move |i| self.traj_buckets[i].trajectories.iter())
    }

    // Returns a copy of trajectories from all buckets within max_angle
    pub fn vec_nearby_angle(&self, angle: f64) -> Vec<Trajectory> {
        let idx: usize = self.angle_to_bucket(angle);

        let u_len: usize = self.traj_buckets.len();
        let i_len: isize = u_len as isize;
        let wrap = |i: isize| -> usize { ((i % i_len) + i_len) as usize % u_len };

        // Number of neighboring buckets required on each side to fully cover max_angle
        let bucket_radius: isize = (self.max_angle / self.bucket_size).ceil() as isize;

        // estimate size to avoid reallocations
        let mut total: usize = 0;
        for offset in -bucket_radius..=bucket_radius {
            let i: usize = wrap(idx as isize + offset);
            total += self.traj_buckets[i].trajectories.len();
        }

        let mut result: Vec<Trajectory> = Vec::with_capacity(total);

        for offset in -bucket_radius..=bucket_radius {
            let i: usize = wrap(idx as isize + offset);
            result.extend(self.traj_buckets[i].trajectories.iter().cloned());
        }

        result
    }
    pub fn get_num_trajectories(&self) -> usize {
        self.traj_buckets.iter().map(|b| b.trajectories.len()).sum()
    }

    #[allow(unused)]
    pub fn print_info(&self) {
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
                println!("  {}", traj.print_info());
            }
        }
    }
}
