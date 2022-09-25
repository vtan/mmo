use std::cmp::Ordering;

use web_sys::{Element, Performance, Window};

const MILLISECS_PER_WINDOW: f64 = 1000.0;
const OUTPUT_HEADER: &str = "p50 | p90 | p95 | max | samples";

pub struct FpsCounter {
    performance: Performance,
    output_element: Option<Element>,
    window_started: f64,
    sample_started: f64,
    samples: Vec<f64>,
}

impl FpsCounter {
    pub fn new(window: &Window) -> FpsCounter {
        FpsCounter {
            performance: window.performance().expect("Performance not available"),
            output_element: window.document().and_then(|doc| doc.get_element_by_id("debug")),
            window_started: 0.0,
            sample_started: 0.0,
            samples: vec![],
        }
    }

    pub fn record_start(&mut self) {
        self.sample_started = self.performance.now();
        if self.samples.is_empty() {
            self.window_started = self.sample_started;
        }
    }

    pub fn record_end(&mut self) {
        let sample_ended = self.performance.now();
        self.samples.push(sample_ended - self.sample_started);

        if sample_ended >= self.window_started + MILLISECS_PER_WINDOW {
            self.report();
            self.samples.clear();
        }
    }

    fn report(&mut self) {
        let len = self.samples.len();
        if len > 0 {
            self.samples.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Less));
            let p50 = self.samples[len / 2];
            let p90 = self.samples[len * 90 / 100];
            let p95 = self.samples[len * 95 / 100];
            let max = self.samples[len - 1];
            if let Some(output_element) = &self.output_element {
                let text =
                    &format!("{OUTPUT_HEADER}\n{p50:.1} | {p90:.1} | {p95:.1} | {max:.1} | {len}");
                output_element.set_inner_html(text);
            }
        }
    }
}
