use crate::storage::raw_trajectories::RawTrajectories;

/// Computes a directional correlation factor in [0.0, 1.0].
///
/// - 0.0 → trajectories are perfectly spread across all buckets (no dominant direction)
/// - 1.0 → all trajectories are in a single bucket (one dominant direction)
///
/// Uses a normalized Herfindahl-Hirschman Index (HHI) on bucket shares,
/// weighted by adjacency: each bucket's share is spread 50% to its neighbors
/// so that near-concentrated distributions (adjacent buckets) score high too.
///
/// # Arguments
/// * `raw` - The raw trajectories to analyze
pub fn directional_correlation(raw: &RawTrajectories) -> f64 {
    let total = raw.get_num_trajectories();

    // Edge cases
    if total == 0 {
        return 0.0;
    }

    let n: usize = raw.traj_buckets.len();
    if n == 0 {
        return 0.0;
    }

    // Only one bucket → always 1.0
    if n == 1 {
        return 1.0;
    }

    // Step 1: raw share per bucket (fraction of total trajectories)
    let shares: Vec<f64> = raw
        .traj_buckets
        .iter()
        .map(|b| b.trajectories.len() as f64 / total as f64)
        .collect();

    // Step 2: adjacency-smoothed share
    // Each bucket contributes 50% of its share to itself and 25% to each neighbor.
    // This ensures two adjacent full buckets score near 1.0, not 0.5.
    let smoothed: Vec<f64> = (0..n)
        .map(|i| {
            let prev = (i + n - 1) % n;
            let next = (i + 1) % n;
            shares[i] * 0.50 + shares[prev] * 0.25 + shares[next] * 0.25
        })
        .collect();

    // Step 3: HHI = sum of squared shares
    let hhi: f64 = smoothed.iter().map(|s| s * s).sum();

    // Step 4: normalize to [0, 1]
    // HHI_min (uniform) = 1/n,  HHI_max (all in one) = 1.0
    // After smoothing, HHI_max is slightly below 1.0 (0.5^2 + 2*0.25^2 = 0.375 for n>=3)
    // So we normalize against the actual theoretical min/max.
    let hhi_min = 1.0 / n as f64; // perfectly uniform
    let hhi_max = 0.50_f64.powi(2)       // one bucket fully loaded after smoothing
            + 2.0 * 0.25_f64.powi(2); // = 0.375 for n >= 3

    if (hhi_max - hhi_min).abs() < f64::EPSILON {
        return 0.0;
    }

    ((hhi - hhi_min) / (hhi_max - hhi_min)).clamp(0.0, 1.0)
}
