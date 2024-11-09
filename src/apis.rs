use std::collections::HashMap;
use crate::components::summoner_matches_page::{GetSummonerMatchesResult, MatchesResultInfo};
use crate::consts::{Map, PlatformRoute, Queue};
use crate::models::entities::lol_match_participant::{LolMatchParticipant, LolMatchParticipantMatchesDetailPage, LolSummonerChampionPage};
use crate::models::entities::lol_match_timeline::LolMatchTimeline;
use crate::models::entities::summoner::{LiveGameResult, LiveGameResultParticipant, LiveGameResultParticipantChampionStats, LiveGameResultParticipantRankedStats, LolSummonerEncounterPage, LolSummonerEncounterPageResult, Summoner};
#[cfg(feature = "ssr")]
use crate::models::update::process_match_timeline::process_match_timeline;
#[cfg(feature = "ssr")]
use crate::models::update::summoner_matches::update_summoner_matches;
#[cfg(feature = "ssr")]
use crate::AppState;
#[cfg(feature = "ssr")]
use futures::stream::FuturesUnordered;
#[cfg(feature = "ssr")]
use futures::StreamExt;
use leptos::prelude::{expect_context, use_context, ServerFnError};
use leptos::{server, Params};
use leptos_router::params::Params;
#[cfg(feature = "ssr")]
use riven::models::spectator_v5::CurrentGameInfo;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
#[cfg(feature = "ssr")]
use bigdecimal::ToPrimitive;
#[cfg(feature = "ssr")]
use chrono::DateTime;
#[cfg(feature = "ssr")]
use riven::RiotApi;
#[cfg(feature = "ssr")]
use sqlx::PgPool;
#[cfg(feature = "ssr")]
use crate::models::db::db_model::LolMatchParticipantLiveGameDb;
#[cfg(feature = "ssr")]
use crate::models::db::round_to_2_decimal_places;
#[cfg(feature = "ssr")]
use crate::models::update::summoner_matches::TempSummoner;

#[server]
pub async fn get_summoner(
    platform_type: String,
    summoner_slug: String,
) -> Result<Summoner, ServerFnError> {
    //log!("Server::Fetching summoner: {}", summoner_slug);
    let state = expect_context::<AppState>();
    let db = state.db.clone();
    let platform_route = PlatformRoute::from_region_str(platform_type.as_str()).unwrap();
    let (game_name, tag_line) = Summoner::parse_slug(summoner_slug.as_str()).unwrap();
    if let Ok(summoner) = Summoner::find_by_exact_details(&db, &platform_route, game_name.as_str(), tag_line.as_str()).await {
        Ok(summoner)
    } else {
        let (game_name, tag_line) = Summoner::parse_slug(summoner_slug.as_str()).unwrap();
        leptos_axum::redirect(format!("/{}?game_name={}&tag_line={}", platform_route.as_region_str(), game_name, tag_line).as_str());
        Err(ServerFnError::new("Summoner not found"))
    }
}

#[server]
pub async fn update_summoner(puuid: String, platform_type: String) -> Result<Option<Summoner>, ServerFnError> {
    let platform_route = PlatformRoute::from_region_str(platform_type.as_str()).unwrap();
    let state = expect_context::<AppState>();
    let riot_api = state.riot_api.clone();
    match riot_api.account_v1()
        .get_by_puuid(platform_route.to_riven().to_regional(), puuid.as_str())
        .await {
        Ok(account) => {
            match riot_api.summoner_v4().get_by_puuid(platform_route.to_riven(), account.puuid.as_str()).await {
                Ok(summoner) => {
                    let db = state.db.clone();
                    let puuid = summoner.puuid.clone();
                    let slug = Summoner::generate_slug(&account.game_name.clone().unwrap(), &account.tag_line.clone().unwrap());
                    leptos_axum::redirect(format!("/{}/summoners/{}", platform_route.as_region_str(), slug).as_str());
                    let summoner = Summoner::insert_or_update_account_and_summoner(&db, platform_route, account, summoner).await?;
                    tokio::spawn(async move {
                        let _ = update_summoner_matches(db.clone(), riot_api, puuid, platform_route.to_riven(), 1500).await;
                    });
                    Ok(Some(summoner))
                }
                _ => {
                    Err(ServerFnError::new("Summoner not found"))
                }
            }
        }
        Err(_) => {
            Err(ServerFnError::new("Summoner not found"))
        }
    }
}


#[server]
pub async fn find_summoner(
    game_name: String,
    tag_line: String,
    platform_type: String,
) -> Result<(), ServerFnError> {
    let state = expect_context::<AppState>();
    let db = state.db.clone();
    let platform_route = PlatformRoute::from_region_str(platform_type.as_str()).unwrap();
    let riven_pr = platform_route.to_riven();
    match Summoner::find_by_details(&db, &platform_route, game_name.as_str(), tag_line.as_str()).await {
        Ok(summoner) => {
            // Generate slug for URL
            let slug = summoner.slug();
            let url = format!(
                "/{}/summoners/{}",
                platform_route.as_region_str(),
                slug,
            );
            leptos_axum::redirect(url.as_str());
        }
        Err(_) => {
            let not_found_url = format!(
                "/{}?game_name={}&tag_line={}",
                platform_route.as_region_str(),
                game_name,
                tag_line
            );
            let riot_api = state.riot_api.clone();
            match riot_api
                .account_v1()
                .get_by_riot_id(riven_pr.to_regional(), game_name.as_str(), tag_line.as_str())
                .await
            {
                Ok(Some(account)) => {
                    match riot_api
                        .summoner_v4()
                        .get_by_puuid(riven_pr, account.puuid.as_str())
                        .await
                    {
                        Ok(summoner_data) => {
                            let slug = Summoner::generate_slug(&account.game_name.clone().unwrap(), &account.tag_line.clone().unwrap());
                            let _ = Summoner::insert_or_update_account_and_summoner(
                                &db,
                                platform_route.into(),
                                account,
                                summoner_data,
                            )
                                .await?;
                            // Generate slug for URL

                            let url = format!(
                                "/{}/summoners/{}",
                                platform_route.as_region_str(),
                                slug,
                            );
                            leptos_axum::redirect(url.as_str());
                        }
                        _ => {
                            leptos_axum::redirect(not_found_url.as_str());
                        }
                    }
                }
                _ => {
                    leptos_axum::redirect(not_found_url.as_str());
                }
            }
        }
    }
    Ok(())
}

#[derive(Params, PartialEq, Serialize, Deserialize, Clone, Debug, Default)]
pub struct MatchFiltersSearch {
    pub queue_id: Option<i32>,
    pub champion_id: Option<i32>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

impl MatchFiltersSearch {
    pub fn from_signals(queue_id: Option<String>, champion_id: Option<String>, start_date: Option<String>, end_date: Option<String>) -> Self {
        Self {
            queue_id: queue_id.map(|x| x.parse::<i32>().unwrap_or_default()),
            champion_id: champion_id.map(|x| x.parse::<i32>().unwrap_or_default()),
            start_date,
            end_date,
        }
    }
}


#[server]
pub async fn get_summoner_matches(summoner_id: i32, page_number: i32, filters: Option<MatchFiltersSearch>) -> Result<GetSummonerMatchesResult, ServerFnError> {
    //log!("Server::Fetching matches for summoner_id: {}", summoner_id);
    let state = use_context::<AppState>();
    let state = state.unwrap();
    let db = state.db.clone();
    LolMatchParticipant::get_match_participant_for_matches_page(&db, summoner_id, page_number, filters.unwrap_or_default()).await.map_err(|_| ServerFnError::new("Error fetching matches"))
}

#[server]
pub async fn get_summoner_champions(summoner_id: i32, filters: Option<MatchFiltersSearch>) -> Result<Vec<LolSummonerChampionPage>, ServerFnError> {
    let state = use_context::<AppState>();
    let state = state.unwrap();
    let db = state.db.clone();
    LolMatchParticipant::get_champions_for_summoner(&db, summoner_id, filters.unwrap_or_default()).await.map_err(|_| ServerFnError::new("Error fetching champions"))
}


#[server]
pub async fn get_match_details(match_id: i32, riot_match_id: String, platform: String) -> Result<Vec<LolMatchParticipantMatchesDetailPage>, ServerFnError> {
    let state = use_context::<AppState>();
    let state = state.unwrap();
    let db = state.db.clone();

    let mut details = LolMatchParticipant::get_details(&db, match_id).await.map_err(|_| ServerFnError::new("Error fetching match details"))?;
    let mut match_timelines = LolMatchTimeline::get_match_timeline(&db, match_id).await?;
    if match_timelines.is_empty() {
        process_match_timeline(&db, state.riot_api.clone(), match_id, riot_match_id, platform).await?;
        match_timelines = LolMatchTimeline::get_match_timeline(&db, match_id).await?;
    }
    for detail in details.iter_mut() {
        let match_timeline = match_timelines.iter().find(|x| x.summoner_id == detail.summoner_id).cloned().unwrap();
        detail.items_event_timeline = match_timeline.items_event_timeline;
        detail.skills_timeline = match_timeline.skills_timeline;
    }
    Ok(details)
}


#[server]
pub async fn get_summoner_encounters(summoner_id: i32, page_number: i32, filters: Option<MatchFiltersSearch>, search_summoner: Option<String>) -> Result<LolSummonerEncounterPageResult, ServerFnError> {
    let state = use_context::<AppState>();
    let state = state.unwrap();
    let db = state.db.clone();
    Summoner::get_encounters(&db, summoner_id, page_number, filters.unwrap_or_default(), search_summoner).await.map_err(|_| ServerFnError::new("Error fetching champions"))
}


#[server]
pub async fn get_live_game(puuid: String, platform_type: String) -> Result<LiveGameResult, ServerFnError> {
    let state = use_context::<AppState>();
    let state = state.unwrap();
    let live_cache = state.live_game_cache.clone();
    if let Some(game_data)= live_cache.get_game_data(&puuid){
        Ok(game_data)
    } else {
        let db = state.db.clone();
        let riot_api = state.riot_api.clone();
        let game_data = get_live_game_data(&db, riot_api, puuid, platform_type).await?;
        live_cache.set_game_data(
            game_data.game_id.clone(),
            game_data.participants.iter().map(|x| x.puuid.clone()).collect(),
            game_data.clone()
        );
        Ok(game_data)
    }

}


#[cfg(feature = "ssr")]
pub async fn get_live_game_data(db:&PgPool, riot_api:Arc<RiotApi>,puuid:String, platform_type:String) -> Result<LiveGameResult, ServerFnError>{
    let platform_route = PlatformRoute::from_region_str(platform_type.as_str()).unwrap();
    let riven_pr = platform_route.to_riven();
    match riot_api
        .spectator_v5()
        .get_current_game_info_by_puuid(riven_pr, puuid.as_str())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))? {
        None => {
            Err(ServerFnError::new("No live game found"))
        }
        Some(current_game_info) => {
            let participant_puuids = current_game_info.participants.iter().filter(|x| !x.puuid.clone().unwrap_or_default().is_empty()).map(|x| x.puuid.clone().unwrap()).collect::<Vec<String>>();
            let mut summoner_details = Summoner::find_summoner_live_details_by_puuids(&db, &participant_puuids).await?;
            let puuids_not_found = participant_puuids.iter().filter(|&x| !summoner_details.contains_key(x)).cloned().collect::<Vec<String>>();
            let summoners_accounts_futures = puuids_not_found.iter().map(|puuid| {
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
                let current_participant = current_game_info.participants.iter().find(|x| x.puuid.clone().unwrap_or_default() == account.puuid).unwrap();
                TempSummoner{
                    game_name: account.game_name.clone().unwrap_or_default(),
                    tag_line: account.tag_line.clone().unwrap_or_default(),
                    puuid:account.puuid.clone(),
                    platform: platform_type.clone(),
                    summoner_level: 0,
                    profile_icon_id: current_participant.profile_icon_id as i32,
                    updated_at:  chrono::Utc::now(),
                }
            }).collect::<Vec<_>>();
            if !new_summoners.is_empty(){
                Summoner::bulk_insert(&db, &new_summoners).await?;
                summoner_details.extend(Summoner::find_summoner_live_details_by_puuids(&db , puuids_not_found.as_slice()).await.unwrap())
            }

            let summoner_ids = summoner_details.iter().map(|(k, x)| x.id).collect::<Vec<i32>>();

            let live_game_stats = LolMatchParticipant::get_live_game_stats(&db, &summoner_ids).await?;
            let mut participants = vec![];
            let default_hashmap = HashMap::new();
            for participant in current_game_info.participants{
                let participant_puuid = participant.puuid.clone();
                if participant_puuid.is_none() || participant_puuid.unwrap_or_default().is_empty(){
                    continue;
                }
                let participant_puuid = participant.puuid.clone().unwrap();
                let summoner_detail = summoner_details.get(participant_puuid.as_str()).unwrap();
                let stats = live_game_stats.get(&summoner_detail.id).unwrap_or(&default_hashmap);

                let champion_stats= match stats.get(&(participant.champion_id.0 as i32)){
                    None => {
                        None
                    }
                    Some(champion_stats) => {
                        Some(LiveGameResultParticipantChampionStats{
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


                let (total_wins, total_ranked)  = stats.iter().fold((0,0), |acc, (k,v)| {
                    (acc.0 + v.total_win, acc.1 + v.total_match)
                });

                let ranked_stats= if total_ranked == 0 {
                    None
                } else {
                    Some(LiveGameResultParticipantRankedStats{
                        total_ranked: total_ranked as i32,
                        total_ranked_wins: total_wins as i32,
                        total_ranked_losses: total_ranked as i32 - total_wins as i32,
                        ranked_win_rate: total_wins as f64 / total_ranked as f64,
                    })
                };

                participants.push(LiveGameResultParticipant{
                    puuid: participant_puuid,
                    champion_id: participant.champion_id.0 as i32,
                    summoner_spell1_id: participant.spell1_id as i32,
                    summoner_spell2_id: participant.spell2_id as i32,
                    perk_primary_selection_id: participant.perks.clone().unwrap().perk_ids.first().cloned().unwrap_or_default() as i32,
                    perk_sub_style_id: participant.perks.unwrap().perk_sub_style as i32,
                    game_name: summoner_detail.game_name.clone(),
                    tag_line: summoner_detail.tag_line.clone(),
                    platform: summoner_detail.platform.clone(),
                    summoner_level: summoner_detail.summoner_level,
                    team_id: participant.team_id as i32,
                    ranked_stats,
                    champion_stats,
                })
            }
            Ok(LiveGameResult {
                game_id: format!("{}_{}", current_game_info.game_id, current_game_info.platform_id),
                game_length: current_game_info.game_length,
                game_map: Map::try_from(current_game_info.map_id.0).unwrap().get_static_name().to_string(),
                queue_name: current_game_info.game_queue_config_id.map(|x| Queue::try_from(x.0).unwrap().get_static_name().to_string()).unwrap_or_default(),
                participants
            })
        }
    }
}
