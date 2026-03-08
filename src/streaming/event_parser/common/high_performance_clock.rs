use std::fmt::Debug;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// High-performance clock manager that reduces system call overhead and minimizes latency
#[derive(Debug)]
pub struct HighPerformanceClock {
    /// Base time point (monotonic clock time at program start)
    base_instant: Instant,
    /// UTC timestamp (microseconds) corresponding to the base time point
    base_timestamp_us: i64,
}

impl HighPerformanceClock {
    /// Create a new high-performance clock
    pub fn new() -> Self {
        let mut best_offset = i64::MAX;
        let mut best_instant = Instant::now();
        let mut best_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        // Take 3 samples, select the one with minimum latency
        for _ in 0..3 {
            let instant_before = Instant::now();
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as i64;
            let instant_after = Instant::now();

            let sample_latency = instant_after.duration_since(instant_before).as_nanos() as i64;

            if sample_latency < best_offset {
                best_offset = sample_latency;
                best_instant = instant_before;
                best_timestamp = timestamp;
            }
        }

        Self {
            base_instant: best_instant,
            base_timestamp_us: best_timestamp,
        }
    }

    /// Get current timestamp (microseconds) using monotonic clock, avoiding system calls
    #[inline(always)]
    pub fn now_micros(&self) -> i64 {
        let elapsed = self.base_instant.elapsed();
        self.base_timestamp_us + elapsed.as_micros() as i64
    }

    /// Calculate elapsed time (microseconds) since the specified timestamp
    #[inline(always)]
    pub fn elapsed_micros_since(&self, start_timestamp_us: i64) -> i64 {
        self.now_micros() - start_timestamp_us
    }

    /// Get high-precision nanosecond timestamp
    #[inline(always)]
    pub fn now_nanos(&self) -> i128 {
        let elapsed = self.base_instant.elapsed();
        (self.base_timestamp_us as i128 * 1000) + elapsed.as_nanos() as i128
    }
}

impl Default for HighPerformanceClock {
    fn default() -> Self {
        Self::new()
    }
}

/// Global high-performance clock instance
static HIGH_PERF_CLOCK: std::sync::OnceLock<HighPerformanceClock> =
    std::sync::OnceLock::new();

/// Get current timestamp (microseconds) from global high-performance clock
#[inline(always)]
pub fn get_high_perf_clock() -> i64 {
    let clock = HIGH_PERF_CLOCK.get_or_init(HighPerformanceClock::new);
    clock.now_micros()
}

/// Calculate elapsed time (microseconds) since the specified timestamp
#[inline(always)]
pub fn elapsed_micros_since(start_timestamp_us: i64) -> i64 {
    get_high_perf_clock() - start_timestamp_us
}
