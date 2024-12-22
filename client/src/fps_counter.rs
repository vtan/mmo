use std::cmp::Ordering;

use web_sys::{Performance, Window};

const MILLISECS_PER_WINDOW: f64 = 1000.0;

pub struct FpsCounter {
    pub agg: FpsCounterAgg,
    performance: Performance,
    window_started: f64,
    sample_started: f64,
    samples: Vec<f64>,
}

pub struct FpsCounterAgg {
    pub fps: f32,
    pub median_ms: f32,
    pub max_ms: f32,
}

impl FpsCounter {
    pub fn new(window: &Window) -> FpsCounter {
        FpsCounter {
            agg: FpsCounterAgg { fps: 0.0, median_ms: 0.0, max_ms: 0.0 },
            performance: window.performance().expect("Performance not available"),
            window_started: 0.0,
            sample_started: 0.0,
            samples: vec![],
        }
    }

    pub fn record_start(&mut self) -> f64 {
        let now = self.performance.now();

        if now >= self.window_started + MILLISECS_PER_WINDOW {
            self.report();
            self.samples.clear();
        }

        self.sample_started = now;
        if self.samples.is_empty() {
            if now - self.window_started < 2.0 * MILLISECS_PER_WINDOW {
                // Start the current window at the end of the last window, not now
                self.window_started += MILLISECS_PER_WINDOW;
            } else {
                // But reset if we are too far behind
                self.window_started = now;
            }
        }
        self.sample_started
    }

    pub fn record_end(&mut self) {
        let sample_ended = self.performance.now();
        self.samples.push(sample_ended - self.sample_started);
    }

    fn report(&mut self) {
        let len = self.samples.len();
        if len > 0 {
            self.samples.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Less));
            self.agg = FpsCounterAgg {
                fps: len as f32,
                median_ms: self.samples[len / 2] as f32,
                max_ms: self.samples[len - 1] as f32,
            };
        }
    }
}
