use crate::backend::task_director::Task;
use crate::ssr::SubscriberMap;
use std::future::Future;
use std::pin::Pin;
use itertools::Itertools;
use leptos::logging::log;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{Duration, Instant};

pub struct SummonerUpdatedSenderCleanupTask {
    summoner_updated_sender: Arc<SubscriberMap>,
    cleanup_interval: Duration,
    next_run: Instant,
    running: Arc<AtomicBool>,
}

impl SummonerUpdatedSenderCleanupTask {
    pub fn new(summoner_updated_sender: Arc<SubscriberMap>, cleanup_interval: Duration) -> Self {
        let next_run = Instant::now() + cleanup_interval;
        Self {
            summoner_updated_sender,
            cleanup_interval,
            next_run,
            running: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Task for SummonerUpdatedSenderCleanupTask {
    fn execute(&self) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        let sender = self.summoner_updated_sender.clone();
        Box::pin(async move {
            // Iterate over the summoner_updated_sender map
            let to_remove = sender
                .iter()
                .filter(|entry| entry.value().receiver_count() == 0)
                .map(|entry| *entry.key())
                .collect_vec();
            let to_remove_len = to_remove.len();
            if to_remove_len > 0 {
                for summoner_id in to_remove {
                    sender.remove(&summoner_id);
                }
                log!("Cleaned up {} inactive broadcast channels", to_remove_len);
            }
        })
    }

    fn next_execution(&self) -> Instant {
        self.next_run
    }

    fn update_schedule(&mut self) {
        self.next_run = Instant::now() + self.cleanup_interval;
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    fn set_running(&self, running: bool) {
        self.running.store(running, Ordering::SeqCst);
    }

    fn clone_box(&self) -> Box<dyn Task> {
        Box::new(Self {
            summoner_updated_sender: self.summoner_updated_sender.clone(),
            cleanup_interval: self.cleanup_interval,
            next_run: self.next_run,
            running: self.running.clone(),
        })
    }

    fn name(&self) -> &'static str {
        "SummonerUpdatedSenderCleanupTask"
    }

    fn allow_concurrent(&self) -> bool {
        false // Do not allow concurrent executions
    }
}
