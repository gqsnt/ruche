use common::consts::platform_route::PlatformRoute;
#[cfg(feature = "ssr")]
use crate::utils::{summoner_not_found_url, summoner_url};
use leptos::prelude::*;
use leptos::server;
use leptos::server_fn::codec::Rkyv;

#[server(input = Rkyv)]
pub async fn search_summoner(
    platform_route: PlatformRoute,
    game_name: String,
    tag_line: String,
) -> Result<(), ServerFnError> {
    let state = expect_context::<crate::ssr::AppState>();
    let db = state.db.clone();

    let riven_pr = platform_route.to_riven();
    match ssr::find_summoner_by_game_name_tag_line(
        &db,
        &platform_route,
        game_name.as_ref(),
        tag_line.as_ref(),
    )
    .await
    {
        Ok(summoner) => {
            leptos_axum::redirect(
                summoner_url(
                    platform_route.as_ref(),
                    summoner.game_name.as_str(),
                    summoner.tag_line.as_str(),
                )
                .as_str(),
            );
        }
        Err(_) => {
            let not_found_url = summoner_not_found_url(
                platform_route.as_ref(),
                game_name.as_ref(),
                tag_line.as_ref(),
            );
            let riot_api = state.riot_api.clone();
            match riot_api
                .account_v1()
                .get_by_riot_id(
                    riven_pr.to_regional(),
                    game_name.as_ref(),
                    tag_line.as_ref(),
                )
                .await
            {
                Ok(Some(account)) => {
                    match riot_api
                        .summoner_v4()
                        .get_by_puuid(riven_pr, account.puuid.as_str())
                        .await
                    {
                        Ok(summoner_data) => {
                            let redirect_url = summoner_url(
                                platform_route.as_ref(),
                                account
                                    .game_name
                                    .clone()
                                    .expect("search summoner: account game name not found")
                                    .as_str(),
                                account
                                    .tag_line
                                    .clone()
                                    .expect("search summoner: account tag line not found")
                                    .as_str(),
                            );
                            ssr::insert_or_update_account_and_summoner(
                                &db,
                                platform_route,
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
pub mod ssr {
    use crate::backend::ssr::{AppError, AppResult, Id, PlatformRouteDb};
    use common::consts::platform_route::PlatformRoute;
    use chrono::Utc;

    pub async fn find_summoner_by_game_name_tag_line(
        db: &sqlx::PgPool,
        platform_route: &PlatformRoute,
        game_name: &str,
        tag_line: &str,
    ) -> AppResult<SummonerDb> {
        sqlx::query_as::<_, SummonerDb>(
            r#"
            SELECT id, game_name, tag_line, platform
            FROM summoners
            WHERE game_name like $1
              AND lower(tag_line) like lower($2)
              AND platform = $3
              "#,
        )
        .bind(game_name)
        .bind(tag_line)
        .bind(PlatformRouteDb::from(*platform_route))
        .fetch_one(db)
        .await
        .map_err(|e| e.into())
    }

    #[derive(sqlx::FromRow)]
    pub struct SummonerDb {
        pub id: i32,
        pub game_name: String,
        pub tag_line: String,
        pub platform: PlatformRouteDb,
    }

    pub async fn insert_or_update_account_and_summoner(
        db: &sqlx::PgPool,
        platform_route: PlatformRoute,
        account: riven::models::account_v1::Account,
        summoner: riven::models::summoner_v4::Summoner,
    ) -> AppResult<()> {
        match find_summoner_id_by_puuid(db, platform_route, &summoner.puuid).await {
            Ok(id) => update_summoner_by_id(db, id, platform_route, account, summoner).await,
            Err(_) => insert_summoner(db, platform_route, account, summoner).await,
        }
    }

    async fn find_summoner_id_by_puuid(
        db: &sqlx::PgPool,
        platform_route: PlatformRoute,
        puuid: &str,
    ) -> AppResult<i32> {
        sqlx::query_as::<_, Id>("SELECT id FROM summoners WHERE puuid = $1 and platform = $2")
            .bind(puuid)
            .bind(PlatformRouteDb::from(platform_route))
            .fetch_one(db)
            .await
            .map(|x| x.id)
            .map_err(AppError::from)
    }

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
            .bind(PlatformRouteDb::from(platform_route))
            .bind(Utc::now().naive_utc())
            .bind(id)
            .execute(db)
            .await?;
        Ok(())
    }

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
            .bind(PlatformRouteDb::from(platform_route))
            .bind(summoner.summoner_level as i32)
            .bind(summoner.profile_icon_id)
            .bind(Utc::now().naive_utc())
            .execute(db)
            .await?;
        Ok(())
    }
}
