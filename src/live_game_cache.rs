use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::models::entities::summoner::LiveGameResult;
use tokio::time::sleep;
type GameId = String;
type PUUID = String;


pub struct LiveGameCache {
    game_cache: DashMap<GameId, (LiveGameResult, Instant)>,
    puuid_to_game: DashMap<PUUID, GameId>,
    expiration_duration: Duration,
}

impl LiveGameCache {
    pub fn new(expiration_duration:Duration) -> Self {
        LiveGameCache {
            game_cache: DashMap::new(),
            puuid_to_game: DashMap::new(),
            expiration_duration,
        }
    }

    // Attempt to retrieve game data from the cache
    pub fn get_game_data(&self, puuid: &PUUID) -> Option<LiveGameResult> {
        if let Some(game_id_entry) = self.puuid_to_game.get(puuid) {
            let game_id = game_id_entry.value().clone();
            if let Some(game_entry) = self.game_cache.get(&game_id) {
                let (game_data, timestamp) = game_entry.value();
                let diff = Instant::now() - *timestamp;
                if diff < self.expiration_duration {
                    let mut data = game_data.clone();
                    data.game_length += diff.as_secs() as i64;
                    return Some(data);
                } else {
                    // Data expired, remove from cache
                    self.game_cache.remove(&game_id);
                    self.puuid_to_game.remove(puuid);
                }
            } else {
                // Inconsistent state, remove PUUID mapping
                self.puuid_to_game.remove(puuid);
            }
        }
        None
    }

    // Update the cache with new game data
    pub fn set_game_data(&self, game_id: GameId, puuids: Vec<PUUID>, game_data: LiveGameResult) {
        let now = Instant::now();
        self.game_cache.insert(game_id.clone(), (game_data, now));
        for puuid in puuids {
            self.puuid_to_game.insert(puuid, game_id.clone());
        }
    }
}




pub async fn cache_cleanup_task(cache: Arc<LiveGameCache>, interval: Duration) {
    loop {
        sleep(interval).await;
        let now = Instant::now();

        // Clean up game_cache
        let expired_game_ids: Vec<GameId> = cache
            .game_cache
            .iter()
            .filter_map(|entry| {
                let (_, timestamp) = entry.value();
                if now.duration_since(*timestamp) >= cache.expiration_duration {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();

        for game_id in expired_game_ids {
            cache.game_cache.remove(&game_id);

            // Clean up puuid_to_game mappings for this game_id
            let expired_puuids: Vec<PUUID> = cache
                .puuid_to_game
                .iter()
                .filter_map(|entry| {
                    if entry.value() == &game_id {
                        Some(entry.key().clone())
                    } else {
                        None
                    }
                })
                .collect();

            for puuid in expired_puuids {
                cache.puuid_to_game.remove(&puuid);
            }
        }
    }
}
