use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::pin::Pin;
use std::future::Future;
use tokio::time::{Duration, Instant};
use itertools::Itertools;
use riven::models::spectator_v5::CurrentGameInfo;
use crate::backend::task_director::Task;
use crate::backend::live_game_cache::LiveGameCache;
use crate::backend::server_fns::get_live_game::ssr::{game_info_to_live_game, get_all_participants_live_game_stats};
use crate::sse::Hub;
use crate::utils::{Puuid, RiotMatchId};
use riven::RiotApi;
use sqlx::PgPool;
use common::consts::platform_route::PlatformRoute;
use crate::backend::ssr::{AppResult, PlatformRouteDb};

pub struct HandleLiveGameCacheTask {
    pub db: PgPool,
    pub riot_api: Arc<RiotApi>,
    pub cache: Arc<LiveGameCache>,
    pub hub: Arc<Hub>,
    pub update_interval: Duration,
    pub next_run: Instant,
    pub running: Arc<AtomicBool>,
}

impl HandleLiveGameCacheTask {
    pub fn new(
        db: PgPool,
        riot_api: Arc<RiotApi>,
        cache: Arc<LiveGameCache>,
        hub: Arc<Hub>,
        update_interval: Duration,
    ) -> Self {
        Self {
            db, riot_api, cache, hub,
            update_interval,
            next_run: Instant::now() + update_interval,
            running: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Task for HandleLiveGameCacheTask {
    fn execute(&self) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        let db = self.db.clone();
        let riot_api = self.riot_api.clone();
        let cache = self.cache.clone();
        let hub = self.hub.clone();

        Box::pin(async move {
            // Clés “actives” = toutes celles connues du hub
            let sids: Vec<i32> = hub.topics.iter().map(|e| *e.key()).collect();
            let mut sids_cache= Vec::with_capacity(sids.len());
            for sid in sids{
                sids_cache.push((
                    cache.get_game_data(sid).await.as_ref().map(|g| g.game_id) ,
                    sid
                    ))
            }
            let (none_ids, ig_ids): (Vec<(Option<RiotMatchId>, i32)>, Vec<(Option<RiotMatchId>, i32)>) =
                sids_cache
                    .into_iter()
                    .partition(|(m, _)| m.is_none());

            let none_ids = none_ids.into_iter().map(|(_, id)| id).collect::<Vec<_>>();
            let ig_ids = ig_ids
                .into_iter()
                .map(|(riot_match_id, id)| (riot_match_id.unwrap(), id))
                .into_group_map();

            let (mut summoner_match_id, mut match_id_game_info) =
                fetch_all_game_info(&db, &riot_api, &ig_ids, &none_ids).await;

            // Transitions
            for (sid, new_mid) in summoner_match_id.drain() {
                let prev_mid = cache.summoner_to_match.get(&sid).await;
                match (prev_mid, new_mid) {
                    (Some(p), Some(n)) if p == n => { match_id_game_info.remove(&n); }
                    (Some(_), None) => { cache.clear_game_data(sid).await; hub.set_live_none(sid); }
                    (None, Some(_)) => { hub.bump_live_epoch(sid); }
                    (Some(p), Some(n)) if p != n => { cache.clear_game_data(sid).await; hub.bump_live_epoch(sid); }
                    _ => {}
                }
            }

            // Mise à jour du cache pour les matchs restants
            if !match_id_game_info.is_empty() {
                let (all_participants, live_game_stats) =
                    get_all_participants_live_game_stats(&db, &riot_api, match_id_game_info.values().collect::<Vec<_>>()).await.unwrap();

                for (mid, gi) in match_id_game_info {
                    let (summoner_ids, live) = game_info_to_live_game(mid, gi, &all_participants, &live_game_stats);
                    cache.set_game_data(mid, summoner_ids, live).await;
                }
            }
        })
    }

    fn next_execution(&self) -> Instant { self.next_run }
    fn update_schedule(&mut self) { self.next_run = Instant::now() + self.update_interval; }
    fn is_running(&self) -> bool { self.running.load(Ordering::SeqCst) }
    fn set_running(&self, running: bool) { self.running.store(running, Ordering::SeqCst); }
    fn clone_box(&self) -> Box<dyn crate::backend::task_director::Task> {
        Box::new(Self {
            db: self.db.clone(),
            riot_api: self.riot_api.clone(),
            cache: self.cache.clone(),
            hub: self.hub.clone(),
            update_interval: self.update_interval,
            next_run: self.next_run,
            running: self.running.clone(),
        })
    }
    fn name(&self) -> &'static str { "HandleLiveGameCacheTask" }
    fn allow_concurrent(&self) -> bool { false }
}


pub async fn fetch_all_game_info(
    db: &PgPool,
    riot_api: &Arc<RiotApi>,
    ig_ids: &HashMap<RiotMatchId, Vec<i32>>,
    none_ids: &Vec<i32>,
) -> (
    HashMap<i32, Option<RiotMatchId>>,
    HashMap<RiotMatchId, CurrentGameInfo>,
) {
    let mut inner_none_ids = none_ids.to_owned();
    let ig_first_ids = ig_ids
        .iter()
        .map(|(match_id, ids)| (*ids.iter().next().unwrap(), *match_id))
        .collect::<HashMap<i32, RiotMatchId>>();
    let mut summoner_match_id = HashMap::new();
    let mut match_id_live_game = HashMap::new();
    let mut first_ids = ig_first_ids.keys().copied().collect::<Vec<_>>();

    let puuids = fetch_summoner_puuids_by_ids(db, &first_ids).await.unwrap();

    while !first_ids.is_empty() {
        let five_first = first_ids
            .drain(..std::cmp::min(5, first_ids.len()))
            .collect::<Vec<_>>();
        let live_game_results = futures::future::join_all(
            five_first
                .iter()
                .map(|id| {
                    let (puuid, platform) = puuids.get(id).unwrap();
                    let riot_api = riot_api.clone();
                    let puuid_ = *puuid;
                    let platform_ = *platform;
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

    let puuids = fetch_summoner_puuids_by_ids(db, &inner_none_ids)
        .await
        .unwrap();
    let puuid_to_ids = puuids
        .iter()
        .map(|(id, (puuid, _))| (*puuid, *id))
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
                    let riot_api = riot_api.clone();
                    let puuid_ = *puuid;
                    let platform_ = *platform;
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
                let new_match_id = RiotMatchId::get_live_version(
                    live_game.platform_id.as_str(),
                    live_game.game_id,
                );
                for p in live_game.participants.iter() {
                    if let Some(puuid_str) = &p.puuid {
                        let puuid = Puuid::new(puuid_str.as_str());
                        if let Some(participant_summoner_id) = puuid_to_ids.get(&puuid) {
                            if participant_summoner_id != &summoner_id {
                                summoner_match_id
                                    .insert(*participant_summoner_id, Some(new_match_id));
                                inner_none_ids.retain(|id| *id != *participant_summoner_id);
                            }
                        }
                    }
                }
                summoner_match_id.insert(summoner_id, Some(new_match_id));
                match_id_live_game.insert(new_match_id, live_game);
            } else {
                summoner_match_id.insert(summoner_id, None);
            }
        }
    }

    (summoner_match_id, match_id_live_game)
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
