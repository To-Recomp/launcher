//! One download speed budget, shared by every game download.
//!
//! Each chunk a download reads takes its share of a common timeline: the next one may be read
//! once the previous ones have "used up" their bytes at the chosen rate. Two downloads at once
//! therefore split the budget, and changing it (even to or from unlimited) acts on the next
//! chunk of the downloads already running. A download that has been idle gets no credit: it never
//! catches up with a burst.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

pub struct Limiter {
    /// 0: unlimited.
    bytes_per_second: AtomicU64,
    /// When the bytes reserved so far have been "paid for".
    free_at: Mutex<Option<Instant>>,
}

impl Limiter {
    pub fn new() -> Limiter {
        Limiter {
            bytes_per_second: AtomicU64::new(0),
            free_at: Mutex::new(None),
        }
    }

    pub fn set(&self, bytes_per_second: u64) {
        self.bytes_per_second
            .store(bytes_per_second, Ordering::Relaxed);
        if bytes_per_second == 0 {
            *self.free_at.lock().unwrap() = None;
        }
    }

    /// How long the caller must wait, from `now`, before reading more after `bytes` were read.
    pub fn reserve(&self, bytes: usize, now: Instant) -> Duration {
        let rate = self.bytes_per_second.load(Ordering::Relaxed);
        if rate == 0 {
            return Duration::ZERO;
        }
        let mut free_at = self.free_at.lock().unwrap();
        let start = free_at.map_or(now, |at| at.max(now));
        *free_at = Some(start + Duration::from_secs_f64(bytes as f64 / rate as f64));
        start.saturating_duration_since(now)
    }

    /// Waits as `reserve` says, giving up at once when `cancel` is set.
    pub fn pace(&self, bytes: usize, cancel: &AtomicBool) {
        let mut wait = self.reserve(bytes, Instant::now());
        while !wait.is_zero() && !cancel.load(Ordering::Relaxed) {
            let slice = wait.min(Duration::from_millis(50));
            std::thread::sleep(slice);
            wait -= slice;
        }
    }
}

/// The budget of this process.
pub fn shared() -> &'static Limiter {
    static SHARED: OnceLock<Limiter> = OnceLock::new();
    SHARED.get_or_init(Limiter::new)
}

pub const BYTES_PER_MEGABYTE: f64 = 1_000_000.0;

/// From a limit in MB/s as the settings keep it; 0, a missing or a nonsensical value is unlimited.
pub fn bytes_per_second(megabytes: Option<f64>) -> u64 {
    megabytes
        .filter(|mb| mb.is_finite() && *mb > 0.0)
        .map_or(0, |mb| (mb * BYTES_PER_MEGABYTE).round() as u64)
}
