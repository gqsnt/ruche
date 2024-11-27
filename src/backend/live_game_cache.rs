use crate::utils::{Puuid, RiotMatchId};
use crate::views::summoner_page::summoner_live_page::LiveGame;
use dashmap::DashMap;
use std::time::Duration;
use tokio::time::Instant;

pub struct LiveGameCache {
    pub game_cache: DashMap<RiotMatchId, (LiveGame, Instant)>,
    pub puuid_to_game: DashMap<Puuid, RiotMatchId>,
    pub expiration_duration: Duration,
}

impl LiveGameCache {
    pub fn new(expiration_duration: Duration) -> Self {
        LiveGameCache {
            game_cache: DashMap::new(),
            puuid_to_game: DashMap::new(),
            expiration_duration,
        }
    }

    // Attempt to retrieve game data from the cache
    pub fn get_game_data(&self, puuid: &Puuid) -> Option<LiveGame> {
        if let Some(game_id_entry) = self.puuid_to_game.get(puuid) {
            let game_id = *game_id_entry.value();
            if let Some(game_entry) = self.game_cache.get(&game_id) {
                let (game_data, timestamp) = game_entry.value();
                let diff = Instant::now() - *timestamp;
                if diff < self.expiration_duration {
                    let mut data = game_data.clone();
                    data.game_length += diff.as_secs() as u16;
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
    pub fn set_game_data(&self, game_id: RiotMatchId, puuids: Vec<Puuid>, game_data: LiveGame) {
        let now = Instant::now();
        self.game_cache.insert(game_id, (game_data, now));
        for puuid in puuids {
            self.puuid_to_game.insert(puuid, game_id);
        }
    }
}
