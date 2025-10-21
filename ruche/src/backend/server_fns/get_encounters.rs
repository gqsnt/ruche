use crate::views::summoner_page::summoner_encounters_page::SummonerEncountersResult;
use crate::views::BackEndMatchFiltersSearch;
use leptos::prelude::*;
use leptos::server;
use leptos::server_fn::codec::Bitcode;
#[server(input=Bitcode,output=Bitcode)]
pub async fn get_encounters(
    summoner_id: i32,
    page_number: u16,
    search_summoner: Option<String>,
    filters: Option<BackEndMatchFiltersSearch>,
) -> Result<SummonerEncountersResult, ServerFnError> {
    let state = expect_context::<crate::ssr::AppState>();
    let db = state.db.clone();

    ssr::inner_get_encounters(
        &db,
        summoner_id,
        page_number as i32,
        filters.unwrap_or_default(),
        search_summoner.map(|r| r.to_string()),
    )
    .await
    .map_err(|e| e.to_server_fn_error())
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::backend::ssr::{AppResult, PlatformRouteDb};
    use crate::views::summoner_page::summoner_encounters_page::{
        SummonerEncountersResult, SummonerEncountersSummoner,
    };
    use crate::views::BackEndMatchFiltersSearch;
    use common::consts::queue::Queue;
    use sqlx::{FromRow, PgPool, QueryBuilder};
    use std::collections::HashMap;

    pub async fn inner_get_encounters(
        db: &PgPool,
        summoner_id: i32,
        page: i32,
        filters: BackEndMatchFiltersSearch,
        search_summoner: Option<String>,
    ) -> AppResult<SummonerEncountersResult> {
        let per_page = 40;
        let offset = (page.max(1) - 1) * per_page;

        let start_date = filters.start_date_to_naive();
        let end_date = filters.end_date_to_naive();
        let search_summoner = search_summoner.filter(|s| !s.is_empty());

        let mut query = QueryBuilder::new(
            r#"
            SELECT
                    lmp.summoner_id,
                    COUNT(*) OVER () AS total_count,
                    COUNT(*) AS match_count,
                    COUNT(*) FILTER (WHERE lmp.team_id = tm.team_id) AS with_match_count,
                    SUM(CASE WHEN lmp.team_id = tm.team_id AND tm.won THEN 1 ELSE 0 END) AS with_win_count,
                    SUM(CASE WHEN lmp.team_id != tm.team_id AND tm.won THEN 1 ELSE 0 END) AS vs_win_count
                FROM
                    lol_match_participants lmp
        "#,
        );

        if search_summoner.is_some() {
            query.push(" LEFT JOIN summoners ss on lmp.summoner_id = ss.id ");
        }

        query.push("JOIN lol_match_participants tm ON lmp.lol_match_id = tm.lol_match_id AND tm.summoner_id = ");

        query.push_bind(summoner_id);
        if let Some(champion_id) = filters.champion_id {
            let sql_filter = " AND tm.champion_id = ";
            query.push(sql_filter);
            query.push_bind(champion_id as i32);
        }
        if filters.queue_id.is_some()
            || start_date.is_some()
            || end_date.is_some()
            || search_summoner.is_some()
        {
            query.push(" JOIN lol_matches lm ON lm.id = lmp.lol_match_id ");
        }
        query.push(" WHERE lmp.summoner_id != ");
        query.push_bind(summoner_id);

        if let Some(search_summoner) = search_summoner {
            query.push(" and ss.game_name ilike ");
            query.push_bind(format!("%{}%", search_summoner));
        }

        if let Some(queue_id) = filters.queue_id {
            let sql_filter = " AND lm.queue_id = ";
            query.push(sql_filter);
            query.push_bind(Queue::from(queue_id).to_u16() as i32);
        }

        if let Some(start_date) = start_date {
            let sql_filter = " AND lm.match_end >= ";
            query.push(sql_filter);
            query.push_bind(start_date);
        }
        if let Some(end_date) = end_date {
            let sql_filter = " AND lm.match_end <= ";
            query.push(sql_filter);
            query.push_bind(end_date);
        }
        query.push(" GROUP BY lmp.summoner_id ORDER BY match_count DESC LIMIT 40 OFFSET ");
        query.push_bind(offset);
        let results = query
            .build_query_as::<LolSummonerEncounterModel>()
            .fetch_all(db)
            .await?;

        let summoners_ids = results
            .iter()
            .map(|encounter| encounter.summoner_id)
            .collect::<Vec<_>>();
        let summoners = sqlx::query_as::<_, (i32, String, String, PlatformRouteDb, i32)>(
            r#"
            select id, game_name, tag_line, platform, profile_icon_id
            from summoners
            where id = any($1)
        "#,
        )
        .bind(&summoners_ids)
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|(id, game_name, tag_line, platform, profile_icon_id)| {
            (id, (game_name, tag_line, platform, profile_icon_id))
        })
        .collect::<HashMap<i32, (String, String, PlatformRouteDb, i32)>>();

        let total_pages = if results.is_empty() {
            0
        } else {
            (results.first().unwrap().total_count as f32 / per_page as f32).ceil() as u16
        };
        Ok(SummonerEncountersResult {
            total_pages,
            encounters: results
                .into_iter()
                .map(|encounter| {
                    let (game_name, tag_line, platform, profile_icon_id) = summoners
                        .get(&encounter.summoner_id)
                        .cloned()
                        .expect("Summoner not found");
                    SummonerEncountersSummoner {
                        id: encounter.summoner_id,
                        profile_icon_id: profile_icon_id as u16,
                        match_count: encounter.match_count as u16,
                        with_match_count: encounter.with_match_count as u16,
                        with_win_count: encounter.with_win_count as u16,
                        vs_win_count: encounter.vs_win_count as u16,
                        game_name,
                        tag_line,
                        platform: platform.into(),
                    }
                })
                .collect::<Vec<_>>(),
        })
    }

    #[derive(FromRow)]
    struct LolSummonerEncounterModel {
        pub summoner_id: i32,
        // pub tag_line: String,
        // pub game_name: String,
        // pub platform: String,
        // pub profile_icon_id: i32,
        pub match_count: i64,
        pub total_count: i64,
        pub with_match_count: i64,
        pub with_win_count: i64,
        pub vs_win_count: i64,
    }
}
