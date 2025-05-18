use crate::backend::live_game_cache::LiveGameCache;
use crate::backend::server_fns::get_live_game::ssr::{
    game_info_to_live_game, get_all_participants_live_game_stats,
};
use crate::backend::ssr::{AppResult, PlatformRouteDb};
use crate::backend::task_director::Task;
use crate::ssr::SubscriberMap;
use crate::utils::{Puuid, RiotMatchId, SSEEvent};
use async_trait::async_trait;
use common::consts::platform_route::PlatformRoute;
use itertools::Itertools;
use riven::models::spectator_v5::CurrentGameInfo;
use riven::RiotApi;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use leptos::logging::log;
use tokio::time::{Duration, Instant};

pub struct HandleLiveGameCacheTask {
    pub db: PgPool,
    pub riot_api: Arc<RiotApi>,
    pub cache: Arc<LiveGameCache>,
    pub summoner_updated_sender: Arc<SubscriberMap>,
    pub update_interval: Duration,
    pub next_run: Instant,
    pub running: Arc<AtomicBool>,
}

impl HandleLiveGameCacheTask {
    pub fn new(
        db: PgPool,
        riot_api: Arc<RiotApi>,
        cache: Arc<LiveGameCache>,
        summoner_updated_sender: Arc<SubscriberMap>,
        update_interval: Duration,
    ) -> Self {
        let next_run = Instant::now() + update_interval;
        Self {
            db,
            riot_api,
            cache,
            summoner_updated_sender,
            update_interval,
            next_run,
            running: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[async_trait]
impl Task for HandleLiveGameCacheTask {
    async fn execute(&self) {
        // split the summoner_updated_sender into two groups, none_ids: Vec<i32> and ig_ids: HashMap<RiotMatchId, Vec<i32>>
        let (none_ids, ig_ids): (
            Vec<(Option<RiotMatchId>, i32)>,
            Vec<(Option<RiotMatchId>, i32)>,
        ) = self
            .summoner_updated_sender
            .iter()
            .map(|entry| {
                let summoner_id = *entry.key();
                let match_id = self
                    .cache
                    .summoner_id_to_game
                    .get(&summoner_id)
                    .map(|entry| entry.value().clone());
                (match_id, summoner_id)
            })
            .partition(|(match_id, _)| match_id.is_none());
        let none_ids = none_ids.into_iter().map(|(_, id)| id).collect::<Vec<_>>();
        let ig_ids = ig_ids
            .into_iter()
            .map(|(riot_match_id, id)| (riot_match_id.unwrap(), id))
            .into_group_map();
        let (summoner_match_id, mut match_id_game_info) =
            self.fetch_all_game_info(&ig_ids, &none_ids).await;

        // determine sse events to send
        let mut sse_events = vec![];
        for (summoner_id, match_id) in summoner_match_id {
            let previous_match_id = self
                .cache
                .summoner_id_to_game
                .get(&summoner_id)
                .map(|entry| entry.value().clone());
            match (previous_match_id, match_id) {
                (Some(previous_match_id), Some(match_id)) => {
                    if previous_match_id != match_id {
                        self.cache.clear_game_data(summoner_id);
                        sse_events.push((summoner_id, SSEEvent::LiveGame(Some(1))));
                    } else {
                        match_id_game_info.remove(&match_id);
                    }
                }
                (Some(_), None) => {
                    self.cache.clear_game_data(summoner_id);
                    sse_events.push((summoner_id, SSEEvent::LiveGame(None)));
                }
                (None, Some(_)) => {
                    sse_events.push((summoner_id, SSEEvent::LiveGame(Some(1))));
                }
                (None, None) => {}
            }
        }

        // update cache
        let (all_participants, live_game_stats) = get_all_participants_live_game_stats(
            &self.db,
            &self.riot_api,
            match_id_game_info.values().collect::<Vec<_>>(),
        )
        .await
        .unwrap();
        for (match_id, game_info) in match_id_game_info {
            let (summoner_ids, live_game) =
                game_info_to_live_game(match_id, game_info, &all_participants, &live_game_stats);
            self.cache.set_game_data(match_id, summoner_ids, live_game);
        }

        // send sse events
        for (summoner_id, event) in sse_events {
            if let Some(sender) = self.summoner_updated_sender.get(&summoner_id) {
                match sender.value().send(event) {
                    Ok(_) => {}
                    Err(e) => {
                        log!("Error sending sse event: {:?}", e);
                    }
                }
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
            db: self.db.clone(),
            riot_api: self.riot_api.clone(),
            summoner_updated_sender: self.summoner_updated_sender.clone(),
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

impl HandleLiveGameCacheTask {
    pub async fn fetch_all_game_info(
        &self,
        ig_ids: &HashMap<RiotMatchId, Vec<i32>>,
        none_ids: &Vec<i32>,
    ) -> (
        HashMap<i32, Option<RiotMatchId>>,
        HashMap<RiotMatchId, CurrentGameInfo>,
    ) {
        let mut inner_none_ids = none_ids.clone();
        let ig_first_ids = ig_ids
            .iter()
            .map(|(match_id, ids)| (*ids.iter().next().unwrap(), match_id.clone()))
            .collect::<HashMap<i32, RiotMatchId>>();
        let mut summoner_match_id = HashMap::new();
        let mut match_id_live_game = HashMap::new();
        let mut first_ids = ig_first_ids.keys().map(|a| *a).collect::<Vec<_>>();

        let puuids = fetch_summoner_puuids_by_ids(&self.db, &first_ids)
            .await
            .unwrap();

        while !first_ids.is_empty() {
            let five_first = first_ids
                .drain(..std::cmp::min(5, first_ids.len()))
                .collect::<Vec<_>>();
            let live_game_results = futures::future::join_all(
                five_first
                    .iter()
                    .map(|id| {
                        let (puuid, platform) = puuids.get(id).unwrap();
                        let riot_api = self.riot_api.clone();
                        let puuid_ = puuid.clone();
                        let platform_ = platform.clone();
                        async move {
                            (
                                *id,
                                riot_api
                                    .spectator_v5()
                                    .get_current_game_info_by_puuid(
                                        platform_.to_riven(),
                                        puuid_.as_ref(),
                                    )
                                    .await
                                    .ok()
                                    .flatten(),
                            )
                        }
                    })
                    .collect::<Vec<_>>(),

            )
            .await;
            for (summoner_id, live_game) in live_game_results {
                let previous_match_id = *ig_first_ids.get(&summoner_id).unwrap();
                let mut participants_ids = ig_ids.get(&previous_match_id).unwrap().clone();
                participants_ids.retain(|id| *id != summoner_id);

                if let Some(live_game) = live_game {
                    let new_match_id = RiotMatchId::get_live_version(
                        live_game.platform_id.as_str(),
                        live_game.game_id,
                    );
                    match_id_live_game.insert(new_match_id, live_game);
                    summoner_match_id.insert(summoner_id, Some(new_match_id));
                    if previous_match_id == new_match_id {
                        for id in participants_ids {
                            summoner_match_id.insert(id, Some(new_match_id));
                        }
                    } else {
                        inner_none_ids.extend(participants_ids);
                    }
                } else {
                    summoner_match_id.insert(summoner_id, None);
                    inner_none_ids.extend(participants_ids);
                }
            }
        }

        let puuids = fetch_summoner_puuids_by_ids(&self.db, &inner_none_ids)
            .await
            .unwrap();
        let puuid_to_ids = puuids
            .iter()
            .map(|(id, (puuid, _))| (puuid.clone(), *id))
            .collect::<HashMap<Puuid, i32>>();

        for (match_id, live_game) in &match_id_live_game {
            for p in &live_game.participants {
                if let Some(puuid_str) = &p.puuid {
                    let puuid = Puuid::new(puuid_str.as_str());
                    if let Some(summoner_id) = puuid_to_ids.get(&puuid) {
                        summoner_match_id.insert(*summoner_id, Some(*match_id));
                        inner_none_ids.retain(|id| *id != *summoner_id);
                    }
                }
            }
        }

        while !inner_none_ids.is_empty() {
            let five_first = inner_none_ids
                .drain(..std::cmp::min(5, inner_none_ids.len()))
                .collect::<Vec<_>>();
            let live_game_results = futures::future::join_all(
                five_first
                    .iter()
                    .map(|id| {
                        let (puuid, platform) = puuids.get(id).unwrap();
                        let riot_api = self.riot_api.clone();
                        let puuid_ = puuid.clone();
                        let platform_ = platform.clone();
                        async move {
                            (
                                *id,
                                riot_api
                                    .spectator_v5()
                                    .get_current_game_info_by_puuid(
                                        platform_.to_riven(),
                                        puuid_.as_ref(),
                                    )
                                    .await
                                    .ok()
                                    .flatten(),
                            )
                        }
                    })
                    .collect::<Vec<_>>(),
            )
            .await;

            for (summoner_id, live_game) in live_game_results {
                if let Some(live_game) = live_game {
                    let match_id = RiotMatchId::get_live_version(
                        live_game.platform_id.as_str(),
                        live_game.game_id,
                    );
                    for p in live_game.participants.iter() {
                        if let Some(puuid_str) = &p.puuid {
                            let puuid = Puuid::new(puuid_str.as_str());
                            if let Some(participant_summoner_id) = puuid_to_ids.get(&puuid) {
                                if participant_summoner_id != &summoner_id {
                                    summoner_match_id
                                        .insert(*participant_summoner_id, Some(match_id));
                                    inner_none_ids.retain(|id| *id != *participant_summoner_id);
                                }
                            }
                        }
                    }
                    summoner_match_id.insert(summoner_id, Some(match_id));
                    match_id_live_game.insert(match_id, live_game);
                } else {
                    summoner_match_id.insert(summoner_id, None);
                }
            }
        }

        (summoner_match_id, match_id_live_game)
    }
}

pub async fn fetch_summoner_puuids_by_ids(
    db: &PgPool,
    summoner_ids: &[i32],
) -> AppResult<HashMap<i32, (Puuid, PlatformRoute)>> {
    Ok(sqlx::query_as::<_, (i32, String, PlatformRouteDb)>(
        "
            SELECT id, puuid, platform
            FROM summoners
            WHERE id = ANY($1)
        ",
    )
    .bind(summoner_ids)
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|(id, puuid, platform)| {
        (
            id,
            (Puuid::new(puuid.as_str()), PlatformRoute::from(platform)),
        )
    })
    .collect::<HashMap<i32, (Puuid, PlatformRoute)>>())
}
