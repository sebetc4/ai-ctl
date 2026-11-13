//! Statistics over the repeated measures of a bench run.
//!
//! Ported from llm-ctl v1 (`PromptResult::from_metrics` and `percentile` in
//! its `src/domain/benchmark.rs`), with three changes: an empty series has no
//! summary rather than zeros, the standard deviation is a sample's, and the
//! 99th percentile is given only over enough values.

/// Below this many values, a 99th percentile is the largest value or an
/// interpolation toward it, and is not reported.
pub const P99_MIN_COUNT: usize = 100;

/// The summary of a series of measures.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Summary {
    pub mean: f64,
    pub median: f64,
    pub min: f64,
    /// The standard deviation of a sample (divided by n − 1), since the
    /// repetitions of a run sample a setting's behaviour; `None` for a single
    /// value.
    pub stddev: Option<f64>,
    /// `None` below [`P99_MIN_COUNT`] values.
    pub p99: Option<f64>,
}

impl Summary {
    /// The summary of `values`, non-finite ones left out; `None` when no
    /// value is left.
    pub fn of(values: impl IntoIterator<Item = f64>) -> Option<Self> {
        let mut sorted: Vec<f64> = values.into_iter().filter(|v| v.is_finite()).collect();
        sorted.sort_by(f64::total_cmp);
        let &min = sorted.first()?;
        let count = sorted.len();
        let n = count as f64;
        let mean = sorted.iter().sum::<f64>() / n;
        let stddev = (count > 1).then(|| {
            let squares: f64 = sorted.iter().map(|v| (v - mean).powi(2)).sum();
            (squares / (n - 1.0)).sqrt()
        });
        Some(Self {
            mean,
            median: percentile(&sorted, 50.0),
            min,
            stddev,
            p99: (count >= P99_MIN_COUNT).then(|| percentile(&sorted, 99.0)),
        })
    }
}

/// The `pct` percentile of `sorted`, which is not empty, interpolated
/// linearly between the two nearest ranks.
fn percentile(sorted: &[f64], pct: f64) -> f64 {
    let rank = pct / 100.0 * (sorted.len() - 1) as f64;
    let (lower, upper) = (rank.floor() as usize, rank.ceil() as usize);
    let fraction = rank - lower as f64;
    sorted[lower] * (1.0 - fraction) + sorted[upper] * fraction
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn a_series_is_summarised_whatever_its_order() {
        let summary = Summary::of([50.0, 40.0, 45.0]).unwrap();
        assert!(close(summary.mean, 45.0));
        assert_eq!(summary.median, 45.0);
        assert_eq!(summary.min, 40.0);
        // Sample deviation: squares 25 + 25 + 0 over n - 1 = 2.
        assert!(close(summary.stddev.unwrap(), 5.0));
        assert_eq!(summary.p99, None);
    }

    #[test]
    fn an_empty_series_has_no_summary() {
        assert_eq!(Summary::of([]), None);
        assert_eq!(Summary::of([f64::NAN, f64::INFINITY]), None);
    }

    #[test]
    fn non_finite_values_are_left_out() {
        let summary = Summary::of([f64::NAN, 2.0, f64::NEG_INFINITY, 4.0]).unwrap();
        assert!(close(summary.mean, 3.0));
        assert_eq!(summary.min, 2.0);
    }

    #[test]
    fn a_single_value_has_no_deviation() {
        let summary = Summary::of([42.0]).unwrap();
        assert_eq!((summary.mean, summary.median), (42.0, 42.0));
        assert_eq!(summary.stddev, None);
    }

    #[test]
    fn the_median_of_an_even_series_is_interpolated() {
        assert_eq!(Summary::of([10.0, 20.0, 30.0, 40.0]).unwrap().median, 25.0);
    }

    #[test]
    fn percentiles_interpolate_between_ranks() {
        let sorted = [10.0, 20.0, 30.0, 40.0, 50.0];
        assert_eq!(percentile(&sorted, 0.0), 10.0);
        assert_eq!(percentile(&sorted, 50.0), 30.0);
        assert_eq!(percentile(&sorted, 100.0), 50.0);
        assert!(close(percentile(&sorted, 99.0), 49.6));
        assert_eq!(percentile(&[42.0], 99.0), 42.0);
    }

    #[test]
    fn the_99th_percentile_needs_enough_values() {
        let short: Vec<f64> = (1..P99_MIN_COUNT).map(|v| v as f64).collect();
        assert_eq!(Summary::of(short).unwrap().p99, None);
        let long: Vec<f64> = (1..=P99_MIN_COUNT).map(|v| v as f64).collect();
        // Rank 0.99 × 99 = 98.01, between 99 and 100.
        assert!(close(Summary::of(long).unwrap().p99.unwrap(), 99.01));
    }
}
