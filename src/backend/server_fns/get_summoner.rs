#[cfg(feature = "ssr")]
use leptos::logging::log;
use crate::consts::platform_route::PlatformRoute;
#[cfg(feature = "ssr")]
use crate::utils::{parse_summoner_slug, summoner_not_found_url};
use crate::views::summoner_page::Summoner;
use leptos::prelude::*;
use leptos::server;
use leptos::server_fn::codec::Rkyv;
use crate::utils::{SummonerSlug};
#[cfg(feature = "ssr")]
use crate::utils::FixedToString;


#[server( input=Rkyv,output=Rkyv)]
pub async fn get_summoner(
    platform_route: PlatformRoute,
    summoner_slug: SummonerSlug,
) -> Result<Summoner, ServerFnError> {
    //log!("Server::Fetching summoner: {}", summoner_slug);
    let state = expect_context::<crate::ssr::AppState>();
    let db = state.db.clone();
    let slug = summoner_slug.to_string();

    let (game_name, tag_line) = parse_summoner_slug(slug.as_str());
    match ssr::find_summoner_by_exact_game_name_tag_line(&db, platform_route, game_name.clone(), tag_line.clone()).await {
        Ok(summoner) => {
            Ok(summoner)
        }
        Err(e) => {
            log!("Server::Summoner not found: {}: {:?}", summoner_slug.to_string(), e);
            leptos_axum::redirect(summoner_not_found_url(platform_route.to_string(), game_name, tag_line).as_str());
            e.as_server_fn_error()
        }
    }
}


#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::backend::ssr::{AppResult, PlatformRouteDb};
    use crate::consts::platform_route::PlatformRoute;
    use crate::views::summoner_page::Summoner;
    use crate::DATE_FORMAT;
    use chrono::NaiveDateTime;
    use crate::utils::string_to_fixed_array;

    pub async fn find_summoner_by_exact_game_name_tag_line(
        db: &sqlx::PgPool,
        platform_route: PlatformRoute,
        game_name: String,
        tag_line: String,
    ) -> AppResult<Summoner> {
        sqlx::query_as::<_, SummonerModel>(
            r#"
            SELECT
               ss.id              as id,
               ss.game_name       as game_name,
               ss.tag_line        as tag_line,
               ss.platform        as platform,
               ss.profile_icon_id as profile_icon_id,
               ss.summoner_level  as summoner_level,
               ss.puuid           as puuid,
               ss.updated_at      as updated_at,
               ss.pro_player_slug as pro_slug
            FROM summoners as ss
            WHERE ss.game_name = $1
              AND lower(ss.tag_line) = lower($2)
              AND ss.platform = $3
  "#
        ).bind(game_name)
            .bind(tag_line)
            .bind(PlatformRouteDb::from(platform_route))
            .fetch_one(db)
            .await
            .map(|summoner_db| {
                Summoner {
                    id: summoner_db.id,
                    game_name: string_to_fixed_array::<16>(summoner_db.game_name.as_str()),
                    tag_line: string_to_fixed_array::<5>(summoner_db.tag_line.as_str()),
                    puuid: string_to_fixed_array::<78>(summoner_db.puuid.as_str()),
                    platform: PlatformRoute::from(summoner_db.platform),
                    updated_at: summoner_db.updated_at.format(DATE_FORMAT).to_string(),
                    summoner_level: summoner_db.summoner_level as u16,
                    profile_icon_id: summoner_db.profile_icon_id as u16,
                    pro_slug: summoner_db.pro_slug.map(|s|string_to_fixed_array::<20>(s.as_str())),
                }
            })
            .map_err(|e| e.into())
    }


    #[derive(sqlx::FromRow, Debug)]
    pub struct SummonerModel {
        pub id: i32,
        pub game_name: String,
        pub tag_line: String,
        pub puuid: String,
        pub platform: PlatformRouteDb,
        pub updated_at: NaiveDateTime,
        pub summoner_level: i32,
        pub profile_icon_id: i32,
        pub pro_slug: Option<String>,
    }
}