use crate::views::summoner_page::summoner_encounters_page::SummonerEncounters;
use crate::views::MatchFiltersSearch;
use leptos::prelude::*;
use leptos::server;

#[server]
pub async fn get_encounters(summoner_id: i32, page_number: i32, filters: Option<MatchFiltersSearch>, search_summoner: Option<String>) -> Result<SummonerEncounters, ServerFnError> {
    let state = expect_context::<crate::ssr::AppState>();
    let db = state.db.clone();

    ssr::inner_get_encounters(&db, summoner_id, page_number, filters.unwrap_or_default(), search_summoner).await.map_err(|e| e.to_server_fn_error())
}


#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::backend::ssr::{parse_date, AppResult};
    use crate::views::summoner_page::summoner_encounters_page::{SummonerEncounter, SummonerEncounters};
    use crate::views::MatchFiltersSearch;
    use sqlx::{FromRow, PgPool, QueryBuilder};


    pub async fn inner_get_encounters(
        db: &PgPool,
        summoner_id: i32,
        page: i32,
        filters: MatchFiltersSearch,
        search_summoner: Option<String>,
    ) -> AppResult<SummonerEncounters> {
        let start_date = parse_date(filters.start_date.clone());
        let end_date = parse_date(filters.end_date.clone());
        let per_page = 40;
        let offset = (page.max(1) - 1) * per_page;

        // initial query
        let mut query = QueryBuilder::new(
            r#"
                SELECT
                    ss.id,
                    ss.tag_line,
                    ss.game_name,
                    ss.platform,
                    ss.profile_icon_id,
                    match_count,
                    total_count,
                    with_match_count,
                    with_win_count,
                    vs_match_count,
                    vs_win_count
                FROM
                    summoners AS ss
                        INNER JOIN (
                        SELECT
                            lmp.summoner_id,
                            COUNT(lmp.id)                                                            AS match_count,
                            count(lmp.summoner_id) over ()                                           AS total_count,
                            COUNT(CASE WHEN lmp1.team_id = lmp.team_id THEN lmp.id END)              AS with_match_count,
                            SUM(CASE WHEN lmp1.team_id = lmp.team_id AND lmp.won THEN 1 ELSE 0 END)  AS with_win_count,
                            COUNT(CASE WHEN lmp1.team_id != lmp.team_id THEN lmp.id END)             AS vs_match_count,
                            SUM(CASE WHEN lmp1.team_id != lmp.team_id AND lmp.won THEN 1 ELSE 0 END) AS vs_win_count
                        FROM
                            lol_match_participants AS lmp
                            INNER JOIN (select id, team_id, won, summoner_id, lol_match_id from lol_match_participants) AS lmp1
                                         ON lmp.lol_match_id = lmp1.lol_match_id
                                             AND lmp1.summoner_id =
        "#);
        query.push_bind(summoner_id);
        query.push(r#" WHERE lmp.summoner_id != "#);
        query.push_bind(summoner_id);

        // add inner requests and filters
        query.push(
            r#"
                        AND EXISTS (
                            SELECT 1
                            FROM lol_match_participants AS lmp1
                                INNER JOIN (select id, queue_id, match_end from lol_matches) as lm on lmp1.lol_match_id = lm.id

        "#);


        // conditional join for search_summoner
        if search_summoner.is_some() && !search_summoner.as_ref().unwrap().is_empty() {
            query.push("inner join (select id, game_name from summoners) as s1 on lmp.summoner_id = s1.id");
        }

        query.push(
            r#"
                            WHERE lmp1.lol_match_id = lmp.lol_match_id
                              AND lmp1.summoner_id =
        "#);
        query.push_bind(summoner_id);
        if let Some(search_summoner) = search_summoner {
            if !search_summoner.is_empty() {
                query.push(" AND s1.game_name ILIKE ");
                query.push_bind(format!("%{}%", search_summoner));
            }
        }
        if let Some(champion_id) = filters.champion_id {
            query.push(" AND lmp1.champion_id = ");
            query.push_bind(champion_id);
        }
        if let Some(queue_id) = filters.queue_id {
            query.push(" AND lm.queue_id = ");
            query.push_bind(queue_id);
        }
        if let Some(start_date) = start_date {
            query.push(" AND lm.match_end >= ");
            query.push_bind(start_date);
        }
        if let Some(end_date) = end_date {
            query.push(" AND lm.match_end <= ");
            query.push_bind(end_date);
        }

        query.push(
            r#"
                            )
                        GROUP BY
                            lmp.summoner_id
                        ORDER BY
                            match_count DESC
                        LIMIT
        "#);
        query.push_bind(per_page);
        query.push(" OFFSET ");
        query.push_bind(offset);
        query.push(" ) AS encounter_data ON ss.id = encounter_data.summoner_id ORDER BY match_count DESC");

        let results = query.build_query_as::<LolSummonerEncounterModel>().fetch_all(db).await?;
        let total_pages = if results.is_empty() {
            0
        } else {
            (results.first().unwrap().total_count as f64 / per_page as f64).ceil() as i64
        };
        Ok(SummonerEncounters {
            total_pages,
            encounters: results.into_iter().map(|encounter| {
                SummonerEncounter {
                    id: encounter.id,
                    profile_icon_id: encounter.profile_icon_id as u16,
                    match_count: encounter.match_count,
                    with_match_count: encounter.with_match_count,
                    with_win_count: encounter.with_win_count,
                    vs_match_count: encounter.vs_match_count,
                    vs_win_count: encounter.vs_win_count,
                    game_name: encounter.game_name,
                    tag_line: encounter.tag_line,
                    platform: encounter.platform,
                }
            }).collect::<Vec<_>>(),
        })
    }


    #[derive(FromRow)]
    struct LolSummonerEncounterModel {
        pub id: i32,
        pub tag_line: String,
        pub game_name: String,
        pub platform: String,
        pub profile_icon_id: i32,
        pub match_count:i64,
        pub total_count:i64,
        pub with_match_count:i64,
        pub with_win_count:i64,
        pub vs_match_count:i64,
        pub vs_win_count:i64,
    }
}
