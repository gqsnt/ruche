use std::{sync::Arc, time::Duration};
use moka::future::{Cache, FutureExt};
use crate::utils::RiotMatchId;
use crate::views::summoner_page::summoner_live_page::LiveGame;

#[derive(Clone)]
pub struct LiveGameWithIdx {
    pub live: LiveGame,
    pub participants: Arc<[i32]>,
}

pub struct LiveGameCache {
    pub(crate) summoner_to_match: Cache<i32, RiotMatchId>,
    match_to_live: Cache<RiotMatchId, Arc<LiveGameWithIdx>>,
}

impl LiveGameCache {
    pub fn new() -> Self {
        let summoner_to_match = Cache::builder()
            .max_capacity(1_000_000)
            .time_to_idle(Duration::from_secs(120))
            .time_to_live(Duration::from_secs(60 * 60))
            .build();

        let summ_ref = summoner_to_match.clone();
        let match_to_live = Cache::builder()
            .max_capacity(50_000)
            .time_to_idle(Duration::from_secs(120))
            .time_to_live(Duration::from_secs(60 * 60))
            .async_eviction_listener(move |mid, v: Arc<LiveGameWithIdx>, _cause| {
                let summ = summ_ref.clone();
                let midc = *mid;
                let ids = v.participants.clone();
                async move {
                    for sid in ids.iter() {
                        if let Some(cur) = summ.get(sid).await {
                            if cur == midc {
                                summ.invalidate(sid).await;
                            }
                        }
                    }
                }.boxed()
            })
            .build();

        Self { summoner_to_match, match_to_live }
    }

    pub async fn get_game_data(&self, summoner_id: i32) -> Option<Arc<LiveGame>> {
        let Some(mid) = self.summoner_to_match
            .get(&summoner_id)
            .await else {
            return None;
        };
        self.match_to_live.get(&mid).await.map(|x| Arc::new(x.live.clone()))
    }

    pub async fn set_game_data(&self, match_id: RiotMatchId, summoner_ids: Vec<i32>, live: LiveGame) {
        let participants: Arc<[i32]> = summoner_ids.clone().into();
        let with_idx = Arc::new(LiveGameWithIdx { live, participants });
        self.match_to_live.insert(match_id, with_idx).await;

        for sid in summoner_ids {
            if let Some(prev) = self.summoner_to_match.get(&sid).await {
                if prev != match_id {
                    self.summoner_to_match.invalidate(&sid).await;
                }
            }
            self.summoner_to_match.insert(sid, match_id).await;
        }
    }

    pub async fn clear_game_data(&self, summoner_id: i32) {
        if let Some(mid) = self.summoner_to_match.get(&summoner_id).await {
            self.summoner_to_match.invalidate(&summoner_id).await;
            if let Some(v) = self.match_to_live.get(&mid).await {
                let mut any = false;
                for sid in v.participants.iter() {
                    if *sid != summoner_id {
                        if let Some(m) = self.summoner_to_match.get(sid).await {
                            if m == mid { any = true; break; }
                        }
                    }
                }
                if !any { self.match_to_live.invalidate(&mid).await; }
            }
        }
    }

}
