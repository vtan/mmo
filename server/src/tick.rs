use std::time::Duration;

use tokio::{
    sync::broadcast,
    task::JoinHandle,
    time::{Instant, MissedTickBehavior},
};

#[derive(Debug, Clone, Copy)]
pub struct Tick {
    pub tick: u64,
    pub monotonic_time: Instant,
}

pub type Sender = broadcast::Sender<Tick>;
pub type Receiver = broadcast::Receiver<Tick>;

pub static TICK_INTERVAL: Duration = Duration::from_millis(100);

pub fn spawn_producer() -> (broadcast::Sender<Tick>, JoinHandle<()>) {
    let (tick_sender, _) = broadcast::channel(8);
    let spawn_tick_sender = tick_sender.clone();
    let join_handle = tokio::spawn(async move {
        let mut tick = 0;
        let mut interval = tokio::time::interval(TICK_INTERVAL);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            let monotonic_time = interval.tick().await;

            tick += 1;
            let tick = Tick { tick, monotonic_time };

            // Ignore errors if there are no receivers
            let _ = spawn_tick_sender.send(tick);
        }
    });
    (tick_sender, join_handle)
}
