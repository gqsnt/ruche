#[cfg(feature = "ssr")]
use crate::backend::server_fns::get_encounter::ssr::find_summoner_puuid_by_id;
#[cfg(feature = "ssr")]
use crate::utils::Puuid;
use crate::views::summoner_page::summoner_live_page::LiveGame;
use common::consts::platform_route::PlatformRoute;
use leptos::prelude::*;
use leptos::server;
use leptos::server_fn::codec::Bitcode;

#[server(input=Bitcode,output=Bitcode)]
pub async fn get_live_game(
    summoner_id: i32,
    platform_route: PlatformRoute,
    force_refresh: bool,
) -> Result<Option<LiveGame>, ServerFnError> {
    let state = expect_context::<crate::ssr::AppState>();
    let live_cache = state.live_game_cache.clone();
    let db = state.db.clone();
    if force_refresh || live_cache.get_game_data(summoner_id).is_none() {
        let riot_api = state.riot_api.clone();
        let puuid = Puuid::new(find_summoner_puuid_by_id(&db, summoner_id).await?.as_str());
        let live_game = ssr::get_live_game_data(&db, &riot_api, puuid, platform_route).await?;
        Ok(match live_game {
            Some((summoner_ids, live_data)) => {
                live_cache.set_game_data(live_data.game_id, summoner_ids, live_data.clone());
                Some(ssr::add_encounters(&db, live_data, summoner_id).await?)
            }
            None => None,
        })
    } else {
        Ok(Some(
            ssr::add_encounters(
                &db,
                live_cache.get_game_data(summoner_id).unwrap(),
                summoner_id,
            )
            .await?,
        ))
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::backend::ssr::{AppResult, PlatformRouteDb};
    use crate::backend::tasks::update_matches::bulk_summoners::bulk_insert_summoners;
    use crate::backend::tasks::update_matches::{SummonerFull, TempSummoner};

    use crate::backend::server_fns::get_matches::ssr::get_summoner_encounters;
    use crate::ssr::RiotApiState;
    use crate::utils::{ProPlayerSlug, Puuid, RiotMatchId};
    use crate::views::summoner_page::summoner_live_page::{
        LiveGame, LiveGameParticipant, LiveGameParticipantChampionStats,
        LiveGameParticipantRankedStats,
    };
    use bigdecimal::{BigDecimal, ToPrimitive};
    use common::consts::map::Map;
    use common::consts::platform_route::PlatformRoute;
    use common::consts::queue::Queue;
    use itertools::Itertools;
    use riven::models::spectator_v5::CurrentGameInfo;
    use riven::RiotApi;
    use sqlx::PgPool;
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;

    pub async fn add_encounters(
        db: &PgPool,
        mut game_data: LiveGame,
        summoner_id: i32,
    ) -> AppResult<LiveGame> {
        let summoners_ids = game_data
            .participants
            .iter()
            .map(|x| x.summoner_id)
            .collect::<Vec<i32>>();
        let encounters = get_summoner_encounters(db, summoner_id, &summoners_ids).await?;
        for participant in game_data.participants.iter_mut() {
            if let Some(encounter_count) = encounters.get(&participant.summoner_id) {
                participant.encounter_count = *encounter_count;
            }
        }
        Ok(game_data)
    }

    pub async fn get_live_game_data(
        db: &PgPool,
        riot_api: &RiotApiState,
        puuid: Puuid,
        platform: PlatformRoute,
    ) -> AppResult<Option<(Vec<i32>, LiveGame)>> {
        let live_game = riot_api
            .spectator_v5()
            .get_current_game_info_by_puuid(platform.to_riven(), puuid.as_ref())
            .await
            .ok()
            .flatten();
        if let Some(live_game) = live_game {
            let (all_participants, live_game_stats) =
                get_all_participants_live_game_stats(db, riot_api, vec![&live_game]).await?;
            let (summoner_ids, live_game) = game_info_to_live_game(
                RiotMatchId::get_live_version(live_game.platform_id.as_str(), live_game.game_id),
                live_game,
                &all_participants,
                &live_game_stats,
            );
            Ok(Some((summoner_ids, live_game)))
        } else {
            Ok(None)
        }
    }

    pub async fn find_and_insert_new_summoners(
        db: &PgPool,
        riot_api: &RiotApiState,
        puuids_details: &Vec<(String, PlatformRoute, i64)>,
    ) -> AppResult<HashMap<String, SummonerFull>> {
        let futures = puuids_details
            .iter()
            .map(|(puuid, platform_route, profile_icon_id)| {
                let api = riot_api.clone();
                async move {
                    let riven_pr = platform_route.to_riven();
                    let account = api
                        .account_v1()
                        .get_by_puuid(riven_pr.to_regional(), puuid.as_str())
                        .await;
                    (account, puuid, platform_route, *profile_icon_id)
                }
            });
        let summoners_accounts = futures::future::join_all(futures).await;
        let new_summoners = summoners_accounts
            .into_iter()
            .filter_map(|(account, _, platform_route, profile_icon_id)| {
                account.ok().map(|account| TempSummoner {
                    game_name: account.game_name.clone().unwrap_or_default(),
                    tag_line: account.tag_line.clone().unwrap_or_default(),
                    puuid: account.puuid.clone(),
                    platform: platform_route.to_string(),
                    summoner_level: 0,
                    profile_icon_id: profile_icon_id as u16,
                    updated_at: chrono::Utc::now(),
                })
            })
            .collect::<Vec<_>>();
        if !new_summoners.is_empty() {
            bulk_insert_summoners(db, &new_summoners).await
        } else {
            Ok(HashMap::new())
        }
    }

    pub fn game_info_to_live_game(
        riot_match_id: RiotMatchId,
        game_info: CurrentGameInfo,
        all_participants: &HashMap<String, SummonerFull>,
        live_game_stats: &HashMap<i32, HashMap<i32, ParticipantLiveStats>>,
    ) -> (Vec<i32>, LiveGame) {
        let mut participants = vec![];
        let default_hashmap = HashMap::new();
        let mut summoner_ids = vec![];
        for participant in &game_info.participants {
            let participant_puuid = participant.puuid.clone();
            if participant_puuid.is_none() || participant_puuid.unwrap_or_default().is_empty() {
                continue;
            }
            let participant_puuid = participant
                .puuid
                .clone()
                .expect("participant puuid is empty");
            let summoner_detail = all_participants
                .get(participant_puuid.as_str())
                .expect("summoner not found");
            let stats = live_game_stats
                .get(&summoner_detail.id)
                .unwrap_or(&default_hashmap);
            let champion_stats =
                stats
                    .get(&(participant.champion_id.0 as i32))
                    .map(|champion_stats| LiveGameParticipantChampionStats {
                        total_champion_played: champion_stats.total_match as u16,
                        total_champion_wins: champion_stats.total_win as u16,
                        avg_kills: champion_stats.avg_kills.to_f32().unwrap_or_default(),
                        avg_deaths: champion_stats.avg_deaths.to_f32().unwrap_or_default(),
                        avg_assists: champion_stats.avg_assists.to_f32().unwrap_or_default(),
                    });
            let (total_wins, total_ranked) = stats.iter().fold((0, 0), |acc, (_, v)| {
                (acc.0 + v.total_win, acc.1 + v.total_match)
            });

            let ranked_stats = if total_ranked == 0 {
                None
            } else {
                Some(LiveGameParticipantRankedStats {
                    total_ranked: total_ranked as u16,
                    total_ranked_wins: total_wins as u16,
                })
            };
            let (perk_primary_selection_id, perk_sub_style_id) = match participant.perks.clone() {
                None => (0, 0),
                Some(perks) => {
                    let primary = perks.perk_ids.first().cloned().unwrap_or_default();
                    let sub_style = perks.perk_sub_style;
                    (primary as u16, sub_style as u16)
                }
            };
            summoner_ids.push(summoner_detail.id);
            participants.push(LiveGameParticipant {
                summoner_id: summoner_detail.id,
                champion_id: participant.champion_id.0 as u16,
                summoner_spell1_id: participant.spell1_id as u16,
                summoner_spell2_id: participant.spell2_id as u16,
                perk_primary_selection_id,
                perk_sub_style_id,
                game_name: summoner_detail.game_name.clone(),
                tag_line: summoner_detail.tag_line.clone(),
                platform: summoner_detail.platform.into(),
                summoner_level: summoner_detail.summoner_level as u16,
                team_id: participant.team_id as u16,
                ranked_stats,
                champion_stats,
                encounter_count: 0,
                pro_player_slug: summoner_detail.pro_player_slug.clone(),
            })
        }
        (
            summoner_ids,
            LiveGame {
                game_id: riot_match_id,
                game_length: game_info.game_length as u16,
                game_map: Map::from_id(game_info.map_id.0).unwrap(),
                queue: game_info
                    .game_queue_config_id
                    .map(|x| Queue::from_id_or_custom(x.0))
                    .unwrap(),
                participants,
            },
        )
    }

    pub async fn get_all_participants_live_game_stats(
        db: &PgPool,
        riot_api: &Arc<RiotApi>,
        live_games: Vec<&CurrentGameInfo>,
    ) -> AppResult<(
        HashMap<String, SummonerFull>,
        HashMap<i32, HashMap<i32, ParticipantLiveStats>>,
    )> {
        let participant_puuids_info = live_games
            .iter()
            .map(|game_info| {
                game_info
                    .participants
                    .iter()
                    .filter_map(|x| {
                        if let Some(puuid) = x.puuid.clone() {
                            if puuid.is_empty() {
                                None
                            } else {
                                Some((
                                    puuid,
                                    (
                                        PlatformRoute::try_from(game_info.platform_id.as_str()).unwrap(),
                                        x.profile_icon_id,
                                    ),
                                ))
                            }
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .unique_by(|(puuid, _)| puuid.clone())
            .collect::<HashMap<String, (PlatformRoute, i64)>>();
        let participant_puuids = participant_puuids_info
            .keys()
            .cloned()
            .collect::<HashSet<String>>();
        let summoner_details = find_summoner_live_by_puuids(db, &participant_puuids).await?;
        let puuids_found = summoner_details
            .keys()
            .cloned()
            .collect::<HashSet<String>>();
        let puuids_platform_not_found = participant_puuids
            .difference(&puuids_found)
            .cloned()
            .map(|puuid| {
                let (platform, profile_icon_id) = participant_puuids_info
                    .get(&puuid)
                    .expect("platform not found");
                (puuid, *platform, *profile_icon_id)
            })
            .collect::<Vec<(String, PlatformRoute, i64)>>();
        let inserted_summoners =
            find_and_insert_new_summoners(db, riot_api, &puuids_platform_not_found).await?;
        let all_participants = inserted_summoners
            .into_iter()
            .chain(summoner_details.into_iter())
            .collect::<HashMap<String, SummonerFull>>();
        let all_summoner_ids = all_participants
            .values()
            .map(|x| x.id)
            .collect::<Vec<i32>>();
        Ok((
            all_participants,
            get_summoners_live_stats(db, &all_summoner_ids).await?,
        ))
    }

    pub async fn get_summoners_live_stats(
        db: &PgPool,
        summoner_ids: &[i32],
    ) -> AppResult<HashMap<i32, HashMap<i32, ParticipantLiveStats>>> {
        let query_results = sqlx::query_as::<_, ParticipantLiveStats>(r#"
            select
                summoner_id,
                champion_id,
                count(lmp.lol_match_id) as total_match,
                sum(CASE WHEN won THEN 1 ELSE 0 END) as total_win,
                avg(lmp.kills) as avg_kills,
                avg(lmp.deaths) as avg_deaths,
                avg(lmp.assists) as avg_assists
            from lol_match_participants as lmp
                join lol_matches as lm on lmp.lol_match_id = lm.id
            where lmp.summoner_id = ANY($1) and lm.queue_id = 420 and lm.match_end >= '2024-09-25 12:00:00'
            group by lmp.summoner_id, lmp.champion_id;
        "#)  // 420 is the queue id for ranked solo/duo and 2024-09-25 is the split 3 s14 start date
            .bind(summoner_ids)
            .fetch_all(db)
            .await?;
        let mut nested_map = HashMap::new();

        for participant in query_results {
            // Insert a new HashMap for this summoner_id if it doesn't already exist
            nested_map
                .entry(participant.summoner_id)
                .or_insert_with(HashMap::new)
                // Insert the participant data into the inner HashMap by champion_id
                .insert(participant.champion_id, participant);
        }
        Ok(nested_map)
    }

    pub async fn find_summoner_live_by_puuids(
        db: &PgPool,
        puuids: &HashSet<String>,
    ) -> AppResult<HashMap<String, SummonerFull>> {
        Ok(sqlx::query_as::<_, SummonerLiveModel>(
            r#"
            SELECT
                ss.id as id,
                puuid,
                game_name,
                tag_line,
                platform,
                summoner_level,
                pro_player_slug as pro_slug
            FROM summoners as ss
            WHERE ss.puuid = ANY($1)"#,
        )
        .bind(puuids.iter().collect::<Vec<_>>())
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|x| {
            (
                x.puuid.clone(),
                SummonerFull {
                    id: x.id,
                    game_name: x.game_name,
                    tag_line: x.tag_line,
                    puuid: x.puuid,
                    platform: PlatformRoute::from(x.platform),
                    summoner_level: x.summoner_level,
                    profile_icon_id: 0,
                    pro_player_slug: x.pro_slug.map(|s| ProPlayerSlug::new(s.as_str())),
                },
            )
        })
        .collect::<HashMap<String, SummonerFull>>())
    }

    #[derive(sqlx::FromRow)]
    pub struct ParticipantLiveStats {
        pub summoner_id: i32,
        pub champion_id: i32,
        pub total_match: i64,
        pub total_win: i64,
        pub avg_kills: BigDecimal,
        pub avg_deaths: BigDecimal,
        pub avg_assists: BigDecimal,
    }

    #[derive(sqlx::FromRow)]
    struct SummonerLiveModel {
        pub id: i32,
        pub game_name: String,
        pub tag_line: String,
        pub puuid: String,
        pub platform: PlatformRouteDb,
        pub summoner_level: i32,
        pub pro_slug: Option<String>,
    }
}
