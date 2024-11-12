use crate::views::summoner_page::summoner_live_page::LiveGame;
use leptos::prelude::*;
use leptos::server;

#[server]
pub async fn get_live_game(puuid: String, platform_type: String) -> Result<Option<LiveGame>, ServerFnError> {
    let state = expect_context::<crate::ssr::AppState>();
    let live_cache = state.live_game_cache.clone();

    if let Some(game_data) = live_cache.get_game_data(&puuid) {
        Ok(Some(game_data))
    } else {
        let db = state.db.clone();
        let riot_api = state.riot_api.clone();
        match ssr::get_live_game_data(&db, riot_api, puuid, platform_type).await {
            Ok(live_game) => {
                live_cache.set_game_data(
                    live_game.game_id.clone(),
                    live_game.participants.iter().map(|x| x.puuid.clone()).collect(),
                    live_game.clone(),
                );
                Ok(Some(live_game))
            }
            Err(_) => { Ok(None) }
        }
    }
}


#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::backend::ssr::{AppError, AppResult};
    use crate::backend::updates::update_matches_task::bulk_summoners::bulk_insert_summoners;
    use crate::backend::updates::update_matches_task::TempSummoner;

    use crate::consts::map::Map;
    use crate::consts::platform_route::PlatformRoute;
    use crate::consts::queue::Queue;
    use crate::utils::round_to_2_decimal_places;
    use crate::views::summoner_page::summoner_live_page::{LiveGame, LiveGameParticipant, LiveGameParticipantChampionStats, LiveGameParticipantRankedStats};
    use bigdecimal::{BigDecimal, ToPrimitive};
    use futures::stream::FuturesUnordered;
    use futures::StreamExt;
    use riven::models::spectator_v5::CurrentGameInfo;
    use riven::RiotApi;
    use sqlx::PgPool;
    use std::collections::HashMap;
    use std::sync::Arc;

    pub async fn get_live_game_data(db: &PgPool, riot_api: Arc<RiotApi>, puuid: String, platform_type: String) -> AppResult<LiveGame> {
        let platform_route = PlatformRoute::from(platform_type.as_str());
        let riven_pr = platform_route.to_riven();
        let current_game_info = riot_api
            .spectator_v5()
            .get_current_game_info_by_puuid(riven_pr, puuid.as_str())
            .await?;

        match current_game_info {
            None => {
                Err(AppError::CustomError("No live game found".to_string()))
            }
            Some(current_game_info) => {
                let participant_puuids = current_game_info.participants.iter().filter(|x| !x.puuid.clone().unwrap_or_default().is_empty()).map(|x| x.puuid.clone().expect("puuid not found")).collect::<Vec<String>>();
                let mut summoner_details = find_summoner_live_by_puuids(&db, &participant_puuids).await?;

                let puuids_not_found = participant_puuids.iter().filter(|&x| !summoner_details.contains_key(x)).cloned().collect::<Vec<String>>();
                find_and_insert_new_summoners(&db, riot_api.clone(), &puuids_not_found, platform_type.clone(), &current_game_info).await?;
                let new_summoners = find_summoner_live_by_puuids(&db, &puuids_not_found).await?;
                summoner_details.extend(new_summoners);


                let summoner_ids = summoner_details.iter().map(|(_, x)| x.id).collect::<Vec<i32>>();

                let live_game_stats = get_summoners_live_stats(&db, &summoner_ids).await?;
                let mut participants = vec![];
                let default_hashmap = HashMap::new();
                for participant in current_game_info.participants {
                    let participant_puuid = participant.puuid.clone();
                    if participant_puuid.is_none() || participant_puuid.unwrap_or_default().is_empty() {
                        continue;
                    }
                    let participant_puuid = participant.puuid.clone().expect("participant puuid is empty");
                    let summoner_detail = summoner_details.get(participant_puuid.as_str()).expect("summoner not found");
                    let stats = live_game_stats.get(&summoner_detail.id).unwrap_or(&default_hashmap);

                    let champion_stats = match stats.get(&(participant.champion_id.0 as i32)) {
                        None => {
                            None
                        }
                        Some(champion_stats) => {
                            Some(LiveGameParticipantChampionStats {
                                total_champion_played: champion_stats.total_match as i32,
                                total_champion_wins: champion_stats.total_win as i32,
                                total_champion_losses: champion_stats.total_match as i32 - champion_stats.total_win as i32,
                                champion_win_rate: champion_stats.total_win as f64 / champion_stats.total_match as f64,
                                avg_kills: round_to_2_decimal_places(champion_stats.avg_kills.to_f64().unwrap_or_default()),
                                avg_deaths: round_to_2_decimal_places(champion_stats.avg_deaths.to_f64().unwrap_or_default()),
                                avg_assists: round_to_2_decimal_places(champion_stats.avg_assists.to_f64().unwrap_or_default()),
                            })
                        }
                    };


                    let (total_wins, total_ranked) = stats.iter().fold((0, 0), |acc, (_, v)| {
                        (acc.0 + v.total_win, acc.1 + v.total_match)
                    });

                    let ranked_stats = if total_ranked == 0 {
                        None
                    } else {
                        Some(LiveGameParticipantRankedStats {
                            total_ranked: total_ranked as i32,
                            total_ranked_wins: total_wins as i32,
                            total_ranked_losses: total_ranked as i32 - total_wins as i32,
                            ranked_win_rate: total_wins as f64 / total_ranked as f64,
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

                    participants.push(LiveGameParticipant {
                        puuid: participant_puuid,
                        champion_id: participant.champion_id.0 as u16,
                        summoner_spell1_id: participant.spell1_id as u16,
                        summoner_spell2_id: participant.spell2_id as u16,
                        perk_primary_selection_id,
                        perk_sub_style_id,
                        game_name: summoner_detail.game_name.clone(),
                        tag_line: summoner_detail.tag_line.clone(),
                        platform: summoner_detail.platform.clone(),
                        summoner_level: summoner_detail.summoner_level,
                        team_id: participant.team_id as i32,
                        ranked_stats,
                        champion_stats,
                    })
                }
                Ok(LiveGame {
                    game_id: format!("{}_{}", current_game_info.game_id, current_game_info.platform_id),
                    game_length: current_game_info.game_length,
                    game_map: Map::from(current_game_info.map_id.0).get_static_name().to_string(),
                    queue_name: current_game_info.game_queue_config_id.map(|x| Queue::try_from(x.0).expect("live game: queue id not found").to_str().to_string()).unwrap_or_default(),
                    participants,
                })
            }
        }
    }


    async fn find_and_insert_new_summoners(db: &PgPool, riot_api: Arc<RiotApi>, puuids: &[String], platform_type: String, game_info: &CurrentGameInfo) -> AppResult<()> {
        let platform_route = PlatformRoute::from(platform_type.as_str());
        let riven_pr = platform_route.to_riven();
        let summoners_accounts_futures = puuids.iter().map(|puuid| {
            let api = riot_api.clone();
            let pt = riven_pr.clone();
            async move {
                api.account_v1().get_by_puuid(pt.to_regional(), puuid.as_str()).await
            }
        });
        let summoners_accounts: Vec<_> = FuturesUnordered::from_iter(summoners_accounts_futures)
            .filter_map(|result| async move { result.ok() })
            .collect()
            .await;
        let new_summoners = summoners_accounts.iter().map(|account| {
            let current_participant = game_info.participants.iter().find(|x| x.puuid.clone().unwrap_or_default() == account.puuid).unwrap();
            TempSummoner {
                game_name: account.game_name.clone().unwrap_or_default(),
                tag_line: account.tag_line.clone().unwrap_or_default(),
                puuid: account.puuid.clone(),
                platform: platform_type.clone(),
                summoner_level: 0,
                profile_icon_id: current_participant.profile_icon_id as u16,
                updated_at: chrono::Utc::now(),
            }
        }).collect::<Vec<_>>();
        if !new_summoners.is_empty() {
            bulk_insert_summoners(&db, &new_summoners).await?;
        }
        Ok(())
    }


    async fn get_summoners_live_stats(
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
                inner join (select id, queue_id,match_end  from lol_matches) as lm on lmp.lol_match_id = lm.id
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

    async fn find_summoner_live_by_puuids(db: &PgPool, puuids: &[String]) -> AppResult<HashMap<String, SummonerLiveModel>> {
        Ok(sqlx::query_as::<_, SummonerLiveModel>(
            "SELECT  id,
                    puuid,
                    game_name,
                    tag_line,
                    platform,
                    summoner_level  FROM summoners WHERE puuid = ANY($1)"
        )
            .bind(puuids)
            .fetch_all(db)
            .await?
            .into_iter()
            .map(|x| (x.puuid.clone(), x))
            .collect::<HashMap<String, SummonerLiveModel>>())
    }


    #[derive(sqlx::FromRow)]
    struct ParticipantLiveStats {
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
        pub platform: String,
        pub summoner_level: i64,
    }
}
