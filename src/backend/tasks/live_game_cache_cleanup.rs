use crate::backend::live_game_cache::LiveGameCache;
use crate::backend::task_director::Task;
use crate::utils::{Puuid, RiotMatchId};
use axum::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{Duration, Instant};

pub struct LiveGameCacheCleanupTask {
    pub cache: Arc<LiveGameCache>,
    pub update_interval: Duration,
    pub next_run: Instant,
    pub running: Arc<AtomicBool>,
}

impl LiveGameCacheCleanupTask {
    pub fn new(cache: Arc<LiveGameCache>, update_interval: Duration) -> Self {
        let next_run = Instant::now() + update_interval;
        Self {
            cache,
            update_interval,
            next_run,
            running: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[async_trait]
impl Task for LiveGameCacheCleanupTask {
    async fn execute(&self) {
        let now = Instant::now();

        // Clean up game_cache
        let expired_game_ids: Vec<RiotMatchId> = self
            .cache
            .game_cache
            .iter()
            .filter_map(|entry| {
                let (_, timestamp) = entry.value();
                if now.duration_since(*timestamp) >= self.cache.expiration_duration {
                    Some(*entry.key())
                } else {
                    None
                }
            })
            .collect();

        for game_id in expired_game_ids {
            self.cache.game_cache.remove(&game_id);

            // Clean up puuid_to_game mappings for this game_id
            let expired_puuids: Vec<Puuid> = self
                .cache
                .puuid_to_game
                .iter()
                .filter_map(|entry| {
                    if entry.value() == &game_id {
                        Some(*entry.key())
                    } else {
                        None
                    }
                })
                .collect();

            for puuid in expired_puuids {
                self.cache.puuid_to_game.remove(&puuid);
            }
        }
    }

    fn next_execution(&self) -> Instant {
        self.next_run
    }

    fn update_schedule(&mut self) {
        self.next_run = Instant::now() + self.update_interval;
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    fn set_running(&self, running: bool) {
        self.running.store(running, Ordering::SeqCst);
    }

    fn clone_box(&self) -> Box<dyn Task> {
        Box::new(Self {
            cache: self.cache.clone(),
            update_interval: self.update_interval,
            next_run: self.next_run,
            running: self.running.clone(),
        })
    }

    fn name(&self) -> &'static str {
        "LiveGameCacheCleanupTask"
    }

    fn allow_concurrent(&self) -> bool {
        false // Do not allow concurrent executions
    }
}
