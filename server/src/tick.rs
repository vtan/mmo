use std::{ops::Sub, time::Duration};

use serde::Deserialize;
use tokio::{
    sync::broadcast,
    task::JoinHandle,
    time::{Instant, MissedTickBehavior},
};

#[derive(Debug, Clone, Copy)]
pub struct TickEvent {
    pub tick: Tick,
    pub monotonic_time: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tick(pub u32);

impl Tick {
    pub fn is_nth(self, rate: TickRate) -> bool {
        self.0 % rate.0 == 0
    }
}

impl Sub<Tick> for Tick {
    type Output = TickDuration;

    fn sub(self, rhs: Tick) -> Self::Output {
        TickDuration(self.0 - rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(from = "f32")]
pub struct TickDuration(pub u32);

impl TickDuration {
    pub fn as_secs_f32(&self) -> f32 {
        self.0 as f32 * TICK_INTERVAL.as_secs_f32()
    }
}

impl From<f32> for TickDuration {
    fn from(secs: f32) -> Self {
        Self((secs / TICK_INTERVAL.as_secs_f32()) as u32)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(from = "f32")]
pub struct TickRate(pub u32);

impl From<f32> for TickRate {
    fn from(secs: f32) -> Self {
        Self((secs / TICK_INTERVAL.as_secs_f32()) as u32)
    }
}

pub type Sender = broadcast::Sender<TickEvent>;
pub type Receiver = broadcast::Receiver<TickEvent>;

pub static TICK_INTERVAL: Duration = Duration::from_millis(100);

pub fn spawn_producer() -> (broadcast::Sender<TickEvent>, JoinHandle<()>) {
    let (tick_sender, _) = broadcast::channel(8);
    let spawn_tick_sender = tick_sender.clone();
    let join_handle = tokio::spawn(async move {
        let mut tick = Tick(0);
        let mut interval = tokio::time::interval(TICK_INTERVAL);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            let monotonic_time = interval.tick().await;

            tick.0 += 1;
            let tick = TickEvent {
                tick,
                monotonic_time,
            };

            // Ignore errors if there are no receivers
            let _ = spawn_tick_sender.send(tick);
        }
    });
    (tick_sender, join_handle)
}
