//! Conversion scheduler for delayed or timed batch processing
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, Instant};

/// Schedules batch conversions for delayed or timed execution.
pub struct ConversionScheduler {
    active: Arc<AtomicBool>,
    scheduled_time: Option<Instant>,
    cancel_flag: Arc<AtomicBool>,
}

impl ConversionScheduler {
    /// Creates a new ConversionScheduler with no active jobs.
    pub fn new() -> Self {
        Self {
            active: Arc::new(AtomicBool::new(false)),
            scheduled_time: None,
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Schedules a callback to execute after the specified delay in seconds.
    pub fn schedule(&mut self, delay_secs: u64, callback: impl FnOnce() + Send + 'static) {
        self.cancel();
        self.cancel_flag.store(false, Ordering::Relaxed);

        if delay_secs == 0 {
            callback();
            return;
        }

        self.active.store(true, Ordering::Relaxed);
        self.scheduled_time = Some(Instant::now() + Duration::from_secs(delay_secs));

        let active = Arc::clone(&self.active);
        let cancel = Arc::clone(&self.cancel_flag);

        thread::spawn(move || {
            let mut elapsed = 0u64;
            while elapsed < delay_secs {
                if cancel.load(Ordering::Relaxed) {
                    active.store(false, Ordering::Relaxed);
                    return;
                }
                thread::sleep(Duration::from_secs(1));
                elapsed += 1;
            }
            active.store(false, Ordering::Relaxed);
            callback();
        });
    }

    /// Cancels any pending scheduled conversion.
    pub fn cancel(&mut self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
        self.active.store(false, Ordering::Relaxed);
        self.scheduled_time = None;
    }

    /// Returns the current scheduler state as a JSON value,
    /// including the remaining time when a conversion is scheduled.
    pub fn get_status(&self) -> serde_json::Value {
        if !self.active.load(Ordering::Relaxed) {
            return serde_json::json!({"active": false});
        }

        if let Some(target) = self.scheduled_time {
            let remaining = target.duration_since(Instant::now()).as_secs();
            let hours = remaining / 3600;
            let mins = (remaining % 3600) / 60;
            let secs = remaining % 60;

            serde_json::json!({
                "active": true,
                "remaining_seconds": remaining,
                "remaining_formatted": format!("{:02}:{:02}:{:02}", hours, mins, secs),
            })
        } else {
            serde_json::json!({"active": false})
        }
    }
}
