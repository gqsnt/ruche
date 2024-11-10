#[cfg(feature = "ssr")]
use crate::backend::server_fns::search_summoner::insert_or_update_account_and_summoner;
#[cfg(feature = "ssr")]
use crate::backend::Id;
use crate::consts::PlatformRoute;
use crate::error_template::{AppError, AppResult};
use crate::views::summoner_page::Summoner;
#[cfg(feature = "ssr")]
use crate::{summoner_url, AppState};
use leptos::logging::log;
use leptos::prelude::{expect_context, ServerFnError};
use leptos::server;
#[cfg(feature = "ssr")]
use riven::consts::RegionalRoute;
#[cfg(feature = "ssr")]
use riven::RiotApi;
use std::collections::HashSet;
use std::sync::Arc;

#[server]
pub async fn update_summoner(puuid: String, platform_type: String) -> Result<(), ServerFnError> {
    let platform_route = PlatformRoute::from_region_str(platform_type.as_str()).unwrap();
    let state = expect_context::<AppState>();
    let riot_api = state.riot_api.clone();
    let max_matches = state.max_matches;
    match riot_api.account_v1()
        .get_by_puuid(platform_route.to_riven().to_regional(), puuid.as_str())
        .await {
        Ok(account) => {
            match riot_api.summoner_v4().get_by_puuid(platform_route.to_riven(), account.puuid.as_str()).await {
                Ok(summoner) => {
                    let db = state.db.clone();
                    let puuid = summoner.puuid.clone();
                    leptos_axum::redirect(summoner_url(platform_route.as_region_str(), &account.game_name.clone().unwrap(), &account.tag_line.clone().unwrap()).as_str());
                    insert_or_update_account_and_summoner(&db, platform_route, account, summoner).await?;
                    tokio::spawn(async move {
                        match update_summoner_default_matches(db.clone(), riot_api, puuid, platform_route.to_riven(), max_matches).await {
                            Ok(_) => {}
                            Err(e) => {
                                log!("Error updating summoner matches: {}", e);
                            }
                        };
                    });
                    Ok(())
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


#[cfg(feature = "ssr")]
pub async fn update_summoner_default_matches(
    db: sqlx::PgPool,
    api: Arc<RiotApi>,
    puuid: String,
    platform: riven::consts::PlatformRoute,
    max_matches: usize,
) -> AppResult<()> {
    let match_ids = fetch_all_match_ids(&api, platform.to_regional(), &puuid, max_matches).await?;

    // Fetch existing match IDs from the database
    let existing_match_ids: HashSet<String> = sqlx::query_scalar(
        "SELECT match_id FROM lol_matches WHERE match_id = ANY($1)",
    )
        .bind(&match_ids)
        .fetch_all(&db)
        .await?
        .into_iter()
        .collect();

    // Filter out matches that are already saved
    let new_riot_match_ids: Vec<String> = match_ids
        .into_iter()
        .filter(|id| !existing_match_ids.contains(id))
        .collect();

    log!("New {} match ids for puuid {}", new_riot_match_ids.len(), puuid);
    //let t = std::time::Instant::now();
    if !new_riot_match_ids.is_empty() {
        bulk_insert_default_match(&db, &new_riot_match_ids).await;
    }
    Ok(())
}


#[cfg(feature = "ssr")]
async fn fetch_all_match_ids(
    api: &RiotApi,
    region: RegionalRoute,
    puuid: &str,
    max_matches: usize,
) -> AppResult<Vec<String>> {
    let max_fetch_limit = max_matches.min(100);
    let mut matches_list = Vec::new();
    let mut begin_index = 0;

    loop {
        let fetched_matches = fetch_match_ids(api, region, puuid, Some(max_fetch_limit as i32), Some(begin_index)).await?;
        if fetched_matches.is_empty() {
            break;
        }
        begin_index += fetched_matches.len() as i32;
        matches_list.extend(fetched_matches);
        if max_matches != 0 && matches_list.len() >= max_matches {
            matches_list.truncate(max_matches);
            break;
        }
    }

    Ok(matches_list)
}


#[cfg(feature = "ssr")]
async fn fetch_match_ids(
    api: &RiotApi,
    region: RegionalRoute,
    puuid: &str,
    count: Option<i32>,
    start: Option<i32>,
) -> AppResult<Vec<String>> {
    api.match_v5()
        .get_match_ids_by_puuid(region, puuid, count, None, None, None, start, None)
        .await
        .map_err(AppError::from)
}

#[cfg(feature = "ssr")]
pub async fn bulk_insert_default_match(db: &sqlx::PgPool, match_ids: &[String]) -> Vec<i32> {
    let match_ids = match_ids.iter().map(|x| x.clone()).collect::<Vec<String>>();
    let platforms = match_ids.iter().map(|x| {
        let match_id_split = x.split("_").collect::<Vec<&str>>();
        match_id_split[0].to_string()
    }).collect::<Vec<String>>();
    let sql = r"
        INSERT INTO
            lol_matches
            (match_id, platform)
        SELECT * FROM UNNEST(
            $1::VARCHAR(17)[],
            $2::VARCHAR(4)[]
        )
        ON CONFLICT (match_id) DO NOTHING
        RETURNING id;
        ";
    let rows = sqlx::query_as::<_, Id>(sql)
        .bind(match_ids)
        .bind(platforms)
        .fetch_all(db)
        .await
        .unwrap();
    rows.into_iter().map(|r| r.id).collect()
}

