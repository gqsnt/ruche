#[cfg(feature = "ssr")]
use crate::backend::Id;
use crate::consts::PlatformRoute;
use crate::error_template::{AppError, AppResult};
#[cfg(feature = "ssr")]
use crate::{summoner_not_found_url, summoner_url, AppState};
#[cfg(feature = "ssr")]
use chrono::Utc;
use leptos::prelude::{expect_context, ServerFnError};
use leptos::server;

#[server]
pub async fn search_summoner(
    game_name: String,
    tag_line: String,
    platform_type: String,
) -> Result<(), ServerFnError> {
    let state = expect_context::<AppState>();
    let db = state.db.clone();
    let platform_route = PlatformRoute::from_region_str(platform_type.as_str()).unwrap();
    let riven_pr = platform_route.to_riven();
    match find_summoner_by_game_name_tag_line(&db, &platform_route, game_name.as_str(), tag_line.as_str()).await {
        Ok(summoner) => {
            leptos_axum::redirect(summoner_url(platform_route.as_region_str(), &summoner.game_name, &summoner.tag_line).as_str());
        }
        Err(_) => {
            let not_found_url = summoner_not_found_url(platform_route.as_region_str(), game_name.as_str(), tag_line.as_str());
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
                            let redirect_url = summoner_url(platform_route.as_region_str(), &account.game_name.clone().unwrap(), &account.tag_line.clone().unwrap());
                            insert_or_update_account_and_summoner(
                                &db,
                                platform_route.into(),
                                account,
                                summoner_data,
                            )
                                .await?;
                            // Generate slug for URL
                            leptos_axum::redirect(redirect_url.as_str());
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

#[cfg(feature = "ssr")]
async fn find_summoner_by_game_name_tag_line(
    db: &sqlx::PgPool,
    platform_route: &PlatformRoute,
    game_name: &str,
    tag_line: &str,
) -> AppResult<SummonerDb> {
    sqlx::query_as::<_, SummonerDb>(
        "SELECT id,game_name,tag_line, platform FROM summoners WHERE game_name ILIKE $1 AND tag_line ILIKE $2 AND platform = $3"
    )
        .bind(game_name)
        .bind(tag_line)
        .bind(platform_route.as_region_str())

        .fetch_one(db)
        .await.map_err(AppError::from)
}

#[cfg(feature = "ssr")]
#[derive(sqlx::FromRow)]
struct SummonerDb {
    pub id: i32,
    pub game_name: String,
    pub tag_line: String,
    pub platform: String,
}

#[cfg(feature = "ssr")]
pub async fn insert_or_update_account_and_summoner(
    db: &sqlx::PgPool,
    platform_route: PlatformRoute,
    account: riven::models::account_v1::Account,
    summoner: riven::models::summoner_v4::Summoner,
) -> AppResult<()> {
    match find_summoner_id_by_puuid(db, platform_route, &summoner.puuid).await {
        Ok(id) => {
            update_summoner_by_id(db, id, platform_route, account, summoner).await
        }
        Err(_) => {
            insert_summoner(db, platform_route, account, summoner).await
        }
    }
}

#[cfg(feature = "ssr")]
async fn find_summoner_id_by_puuid(db: &sqlx::PgPool, platform_route: PlatformRoute, puuid: &str) -> AppResult<i32> {
    sqlx::query_as::<_, Id>("SELECT id FROM summoners WHERE puuid = $1 and platform = $2")
        .bind(puuid)
        .bind(platform_route.as_region_str())
        .fetch_one(db)
        .await
        .map(|x| x.id)
        .map_err(AppError::from)
}

#[cfg(feature = "ssr")]
async fn update_summoner_by_id(
    db: &sqlx::PgPool,
    id: i32,
    platform_route: PlatformRoute,
    account: riven::models::account_v1::Account,
    summoner: riven::models::summoner_v4::Summoner,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE summoners SET game_name = $1, tag_line = $2, puuid = $3, summoner_level = $4, profile_icon_id = $5, platform = $6, updated_at = NOW() WHERE id = $8"
    )
        .bind(account.game_name.clone())
        .bind(account.tag_line.clone())
        .bind(summoner.puuid.clone())
        .bind(summoner.summoner_level as i32)
        .bind(summoner.profile_icon_id)
        .bind(platform_route.as_region_str())
        .bind(Utc::now().naive_utc())
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

#[cfg(feature = "ssr")]
async fn insert_summoner(
    db: &sqlx::PgPool,
    platform_route: PlatformRoute,
    account: riven::models::account_v1::Account,
    summoner: riven::models::summoner_v4::Summoner,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO summoners(game_name, tag_line, puuid, platform, summoner_level, profile_icon_id, updated_at) VALUES ($1, $2, $3, $4, $5, $6, NOW())"
    )
        .bind(account.game_name.clone())
        .bind(account.tag_line.clone())
        .bind(summoner.puuid.clone())
        .bind(platform_route.as_region_str())
        .bind(summoner.summoner_level as i32)
        .bind(summoner.profile_icon_id)
        .bind(Utc::now().naive_utc())
        .execute(db)
        .await?;
    Ok(())
}