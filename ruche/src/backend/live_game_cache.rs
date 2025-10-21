use crate::utils::RiotMatchId;
use crate::views::summoner_page::summoner_live_page::LiveGame;
use dashmap::DashMap;
use std::time::Duration;
use tokio::time::Instant;

pub struct LiveGameCache {
    pub game_cache: DashMap<RiotMatchId, (LiveGame, Instant)>,
    pub summoner_id_to_game: DashMap<i32, RiotMatchId>,
    pub expiration_duration: Duration,
}

impl LiveGameCache {
    pub fn new(expiration_duration: Duration) -> Self {
        LiveGameCache {
            game_cache: DashMap::new(),
            summoner_id_to_game: DashMap::new(),
            expiration_duration,
        }
    }

    pub fn get_game_data(&self, summoner_id: i32) -> Option<LiveGame> {
        let remove_summoner_id = if let Some(game_id_entry) = self.summoner_id_to_game.get(&summoner_id) {
            let game_id = *game_id_entry.value();
            if let Some(game_entry) = self.game_cache.get(&game_id) {
                let (game_data, timestamp) = game_entry.value();
                let diff = Instant::now() - *timestamp;
                let mut data = game_data.clone();
                data.game_length += diff.as_secs() as u16;
                return Some(data);
            } else {
                true
            }
        }else{
            false
        };
        if remove_summoner_id {
            self.summoner_id_to_game.remove(&summoner_id);
        }
        None
    }

    pub fn clear_game_data(&self, summoner_id: i32) -> Vec<i32> {
        let mut summoner_ids = Vec::new();
        let mut game_id = None;
        if let Some(game_id_entry) = self.summoner_id_to_game.get(&summoner_id) {
            let inner_game_id = *game_id_entry.value();
            game_id = Some(inner_game_id);
            if let Some(game_cache) = self.game_cache.get(&inner_game_id){
                let (cache_info, _)  = game_cache.value();
                for participant in cache_info.participants.iter() {
                    summoner_ids.push(participant.summoner_id);
                }
            }

        }
        if let Some(game_id) = game_id {
            self.game_cache.remove(&game_id);
        }
        for summoner_id in summoner_ids.iter() {
            self.summoner_id_to_game.remove(summoner_id);
        }
        summoner_ids
    }

    pub fn set_game_data(&self, game_id: RiotMatchId, summoner_ids: Vec<i32>, game_data: LiveGame) {
        self.game_cache.insert(game_id, (game_data, Instant::now()));
        for summoner_id in summoner_ids {
            self.summoner_id_to_game.insert(summoner_id, game_id);
        }
    }
}
