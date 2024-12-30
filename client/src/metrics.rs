use std::cmp::Ordering;

use web_sys::{Performance, Window};

const MILLISECS_PER_WINDOW: f64 = 1000.0;

#[derive(Debug, Clone)]
pub struct Metrics {
    pub fps_stats: FpsStats,
    pub net_stats: NetStats,
    performance: Performance,
    window_started: f64,
    sample_started: f64,
    samples: Vec<f64>,
    current_net_stats: NetStats,
}

#[derive(Debug, Clone, Default)]
pub struct FpsStats {
    pub fps: f32,
    pub median_ms: f32,
    pub max_ms: f32,
}

#[derive(Debug, Clone, Default)]
pub struct NetStats {
    pub in_bytes: u32,
    pub in_frames: u32,
    pub in_events: u32,
    pub out_bytes: u32,
    pub out_frames: u32,
    pub out_commands: u32,
}

impl Metrics {
    pub fn new(window: &Window) -> Metrics {
        Metrics {
            fps_stats: FpsStats::default(),
            net_stats: NetStats::default(),
            performance: window.performance().expect("Performance not available"),
            window_started: 0.0,
            sample_started: 0.0,
            samples: vec![],
            current_net_stats: NetStats::default(),
        }
    }

    pub fn record_frame_start(&mut self) -> f64 {
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

    pub fn record_frame_end(&mut self) {
        let sample_ended = self.performance.now();
        self.samples.push(sample_ended - self.sample_started);
    }

    pub fn record_net_event(&mut self, len: u32, events: u32) {
        self.current_net_stats.in_frames += 1;
        self.current_net_stats.in_events += events;
        self.current_net_stats.in_bytes += len;
    }

    pub fn record_net_command(&mut self, len: u32, commands: u32) {
        self.current_net_stats.out_frames += 1;
        self.current_net_stats.out_commands += commands;
        self.current_net_stats.out_bytes += len;
    }

    fn report(&mut self) {
        let len = self.samples.len();
        if len > 0 {
            self.samples
                .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Less));
            self.fps_stats = FpsStats {
                fps: len as f32,
                median_ms: self.samples[len / 2] as f32,
                max_ms: self.samples[len - 1] as f32,
            };
        }
        self.net_stats = std::mem::take(&mut self.current_net_stats);
    }
}
