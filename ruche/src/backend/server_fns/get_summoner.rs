#[cfg(feature = "ssr")]
use crate::utils::{summoner_not_found_url};
use crate::views::summoner_page::Summoner;
use leptos::prelude::*;
use leptos::server;
use leptos::server_fn::codec::Bitcode;
use crate::app::SummonerIdentifier;


#[server( input=Bitcode,output=Bitcode)]
pub async fn get_summoner(
    summoner_identifier: SummonerIdentifier
) -> Result<Summoner, ServerFnError> {
    use crate::backend::server_fns::get_summoner::ssr::resolve_summoner_by_s_identifier;
    //log!("Server::Fetching summoner: {}", summoner_slug);
    let state = expect_context::<crate::ssr::AppState>();
    let db = state.db.clone();

    match resolve_summoner_by_s_identifier(&db, &summoner_identifier).await{
        Ok(summoner) => Ok(summoner),
        Err(e) => {
            leptos_axum::redirect(
                summoner_not_found_url(
                    summoner_identifier.platform_route.code(),
                    summoner_identifier.game_name.as_str(),
                    summoner_identifier.tag_line.as_str(),
                )
                    .as_str(),
            );
            Err(e.into())
        },
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use std::sync::Arc;
    use sqlx::PgPool;
    use crate::backend::ssr::{AppError, AppResult, Id, PlatformRouteDb};
    use crate::utils::ProPlayerSlug;
    use crate::views::summoner_page::Summoner;
    use common::consts::platform_route::PlatformRoute;
    use crate::app::SummonerIdentifier;
    use crate::ssr::S_IDENTIFIER_TO_ID;
    
    pub async fn resolve_summoner_by_s_identifier(
        db:&PgPool, summoner_identifier: &SummonerIdentifier,
    ) -> AppResult<Summoner>{
       if let Ok(id) = resolve_id_by_s_identifier(db, summoner_identifier)
           .await{
           Ok(find_summoner_by_id(db,id).await?)
       }else{
           Err(AppError::CustomError("summoner not found".to_string()))
       }
    }
    

    pub async fn resolve_id_by_s_identifier(db:&PgPool, s_identifier:&SummonerIdentifier) -> AppResult<i32> {
        // Single-flight: if N callers hit this concurrently with same slug,
        // loader runs only once; others await the same future.
        let arc_id = S_IDENTIFIER_TO_ID
            .try_get_with(s_identifier.clone(), async move {
                // 1) Fast path: DB lookup by slug
                db_lookup_id_by_s_identifier(db, s_identifier).await?.map(|id|Arc::new(id))
                    .ok_or(AppError::CustomError("summoner not found".to_string()))
            })
            .await
            .map_err(|e|e.as_ref().clone())?; // moka error -> anyhow
        Ok(*arc_id)
    }


    pub async  fn db_lookup_id_by_s_identifier(db:&PgPool, s_identifier:&SummonerIdentifier) -> AppResult<Option<i32>> {
        Ok(sqlx::query_as::<_, Id>(
            r#"
            SELECT
               ss.id              as id
            FROM summoners as ss
            WHERE ss.platform = $1 and ss.game_name = $2
              AND lower(ss.tag_line) = lower($3)
  "#,
        )
            .bind(PlatformRouteDb::from(s_identifier.platform_route))
            .bind(s_identifier.game_name.as_str())
            .bind(s_identifier.tag_line.as_str())
            .fetch_optional(db)
            .await?
            .map(|id|id.id))
    }




    pub async fn find_summoner_by_id(
        db: &sqlx::PgPool,
        summoner_id:i32
    ) -> AppResult<Summoner> {
        Ok(sqlx::query_as::<_, SummonerModel>(
            r#"
            SELECT
               ss.id              as id,
               ss.game_name       as game_name,
               ss.tag_line        as tag_line,
               ss.platform        as platform,
               ss.profile_icon_id as profile_icon_id,
               ss.summoner_level  as summoner_level,
               ss.pro_player_slug as pro_slug
            FROM summoners as ss
            WHERE ss.id = $1
  "#,
        )
        .bind(summoner_id)
        .fetch_one(db)
        .await
        .map(|summoner_db| Summoner {
            id: summoner_db.id,
            game_name: summoner_db.game_name,
            tag_line: summoner_db.tag_line,
            platform: PlatformRoute::from(summoner_db.platform),
            summoner_level: summoner_db.summoner_level as u16,
            profile_icon_id: summoner_db.profile_icon_id as u16,
            pro_slug: summoner_db.pro_slug.map(|s| ProPlayerSlug::new(s.as_str())),
        })?)
    }

    #[derive(sqlx::FromRow, Debug)]
    pub struct SummonerModel {
        pub id: i32,
        pub game_name: String,
        pub tag_line: String,
        pub platform: PlatformRouteDb,
        pub summoner_level: i32,
        pub profile_icon_id: i32,
        pub pro_slug: Option<String>,
    }
}
