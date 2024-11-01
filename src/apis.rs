use std::str::FromStr;
use leptos::context::use_context;
use leptos::prelude::{expect_context, log, ServerFnError};
use leptos::{server, Params};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use crate::AppState;
use crate::components::summoner_matches_page::GetSummonerMatchesResult;
use crate::consts::PlatformRoute;
use crate::models::entities::lol_match_participant::LolMatchParticipant;
use crate::models::entities::summoner::Summoner;
#[cfg(feature = "ssr")]
use crate::models::update::summoner_matches::update_summoner_matches;
use leptos_router::params::Params;


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
    if let Ok(summoner) = Summoner::find_by_details(&db, &platform_route, game_name.as_str(), tag_line.as_str()).await {
        Ok(summoner)
    } else {
        let (game_name, tag_line) = Summoner::parse_slug(summoner_slug.as_str()).unwrap();
        leptos_axum::redirect(format!("/{}?game_name={}&tag_line={}", platform_route.as_region_str(), game_name, tag_line).as_str());
        Err(ServerFnError::new("Summoner not found"))
    }
}

#[server]
pub async fn update_summoner(puuid: String, platform_type: String) -> Result<Option<Summoner>, ServerFnError>{
    let platform_route = PlatformRoute::from_region_str(platform_type.as_str()).unwrap();
    let riven_pr= riven::consts::PlatformRoute::from_str(platform_route.to_string().as_str()).unwrap();
    let state = expect_context::<AppState>();
    let riot_api = state.riot_api.clone();
    match riot_api.account_v1()
        .get_by_puuid(riven_pr.to_regional(), puuid.as_str())
        .await{
        Ok(account) => {
            match riot_api.summoner_v4().get_by_puuid(riven_pr, account.puuid.as_str()).await{
                Ok(summoner) => {
                    let db = state.db.clone();
                    let puuid = summoner.puuid.clone();
                    let slug = Summoner::generate_slug(&account.game_name.clone().unwrap(), &account.tag_line.clone().unwrap());
                    leptos_axum::redirect(format!("/{}/summoners/{}", platform_route.as_region_str(), slug).as_str());
                    let summoner = Summoner::insert_or_update_account_and_summoner(&db, platform_route, account, summoner).await?;
                    tokio::spawn(async move {
                        let _ = update_summoner_matches(db.clone(), riot_api, puuid, riven_pr, 1000).await;
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
    let riven_pr= riven::consts::PlatformRoute::from_str(platform_route.to_string().as_str()).unwrap();

    match Summoner::find_by_slug(&db, &platform_route, game_name.as_str(), tag_line.as_str()).await {
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
                            let summoner = Summoner::insert_or_update_account_and_summoner(
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
    pub queue_id:Option<i32>,
    pub champion_id:Option<i32>,
    pub start_date:Option<String>,
    pub end_date:Option<String>,
}

impl MatchFiltersSearch{
    pub fn from_signals(queue_id:Option<String>, champion_id:Option<String>, start_date:Option<String>, end_date:Option<String>) -> Self{
        Self{
            queue_id: queue_id.map(|x| x.parse::<i32>().unwrap()),
            champion_id: champion_id.map(|x| x.parse::<i32>().unwrap()),
            start_date,
            end_date,
        }
    }
}


#[server]
pub async fn get_summoner_matches(summoner_id:i32, page_number: i32, filters:Option<MatchFiltersSearch>) -> Result<GetSummonerMatchesResult, ServerFnError> {
    //log!("Server::Fetching matches for summoner_id: {}", summoner_id);
    let state = use_context::<AppState>();
    let state = state.unwrap();
    let db = state.db.clone();
    let (matches, total_pages) = LolMatchParticipant::get_match_participant_for_matches_page(&db,summoner_id, page_number, filters.unwrap_or_default()).await.unwrap();
    Ok(GetSummonerMatchesResult { matches, total_pages })
}
