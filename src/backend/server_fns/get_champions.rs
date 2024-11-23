use crate::views::summoner_page::summoner_champions_page::ChampionStats;
use crate::views::{BackEndMatchFiltersSearch};
use leptos::prelude::*;
use leptos::server;
use leptos::server_fn::codec::Rkyv;

#[server(input=Rkyv,output=Rkyv)]
pub async fn get_champions(summoner_id: i32, filters: Option<BackEndMatchFiltersSearch>) -> Result<Vec<ChampionStats>, ServerFnError> {
    let state = expect_context::<crate::ssr::AppState>();
    let db = state.db.clone();

    ssr::inner_get_champions(&db, summoner_id, filters.unwrap_or_default()).await.map_err(|e| e.to_server_fn_error())
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::backend::ssr::AppResult;
    use crate::consts::champion::Champion;
    use crate::utils::round_to_2_decimal_places;
    use crate::views::summoner_page::summoner_champions_page::ChampionStats;
    use crate::views::{BackEndMatchFiltersSearch};
    use bigdecimal::{BigDecimal, ToPrimitive};
    use itertools::Itertools;
    use sqlx::{FromRow, PgPool, QueryBuilder};

    pub async fn inner_get_champions(
        db: &PgPool,
        summoner_id: i32,
        filters: BackEndMatchFiltersSearch,
    ) -> AppResult<Vec<ChampionStats>> {
        let start_date = filters.start_date_to_naive();
        let end_date = filters.end_date_to_naive();


        let mut query = QueryBuilder::new(r#"
             SELECT lmp.champion_id,
               count(lmp.lol_match_id)                  as total_matches,
               sum(CASE WHEN lmp.won THEN 1 ELSE 0 END) AS total_wins,
               avg(lmp.kda)                             as avg_kda,
               avg(lmp.kill_participation)              as avg_kill_participation,
               avg(lmp.kills)                           as avg_kills,
               avg(lmp.deaths)                          as avg_deaths,
               avg(lmp.assists)                         as avg_assists,
               avg(lmp.gold_earned)                     as avg_gold_earned,
               avg(lmp.cs)                              as avg_cs,
               avg(lmp.damage_dealt_to_champions)       as avg_damage_dealt_to_champions,
               avg(lmp.damage_taken)                    as avg_damage_taken,
               sum(lmp.double_kills)                    AS total_double_kills,
               sum(lmp.triple_kills)                    AS total_triple_kills,
               sum(lmp.quadra_kills)                    AS total_quadra_kills,
               sum(lmp.penta_kills)                     AS total_penta_kills
            FROM lol_match_participants as lmp
                     left JOIN lol_matches lm ON lm.id = lmp.lol_match_id
            WHERE lmp.summoner_id =
        "#);

        query.push_bind(summoner_id);
        if let Some(champion_id) = filters.champion_id {
            let sql_filter = " AND lmp.champion_id = ";
            query.push(sql_filter);
            query.push_bind(champion_id);
        }
        if let Some(queue_id) = filters.queue_id {
            let sql_filter = " AND lm.queue_id = ";
            query.push(sql_filter);
            query.push_bind(queue_id);
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
        query.push(" GROUP BY lmp.champion_id ORDER BY total_matches DESC");

        Ok(query.build_query_as::<ChampionStatsModel>()
            .fetch_all(db).await?
            .into_iter()
            .map(|champion_stats| {
                let total_lose = champion_stats.total_matches - champion_stats.total_wins;
                let win_rate = round_to_2_decimal_places((champion_stats.total_wins as f64 / champion_stats.total_matches as f64) * 100.0);
                ChampionStats {
                    champion_id: champion_stats.champion_id as u16,
                    champion_name: Champion::from(champion_stats.champion_id as u16)
                        .to_str().to_string(),
                    total_matches: champion_stats.total_matches,
                    total_wins: champion_stats.total_wins,
                    total_lose,
                    win_rate,
                    avg_kda: round_to_2_decimal_places(champion_stats.avg_kda.to_f64().unwrap_or_default()),
                    avg_kill_participation: round_to_2_decimal_places(champion_stats.avg_kill_participation.to_f64().unwrap_or_default()),
                    avg_kills: round_to_2_decimal_places(champion_stats.avg_kills.to_f64().unwrap_or_default()),
                    avg_deaths: round_to_2_decimal_places(champion_stats.avg_deaths.to_f64().unwrap_or_default()),
                    avg_assists: round_to_2_decimal_places(champion_stats.avg_assists.to_f64().unwrap_or_default()),
                    avg_gold_earned: round_to_2_decimal_places(champion_stats.avg_gold_earned.to_f64().unwrap_or_default()),
                    avg_cs: round_to_2_decimal_places(champion_stats.avg_cs.to_f64().unwrap_or_default()),
                    avg_damage_dealt_to_champions: round_to_2_decimal_places(champion_stats.avg_damage_dealt_to_champions.to_f64().unwrap_or_default()),
                    avg_damage_taken: round_to_2_decimal_places(champion_stats.avg_damage_taken.to_f64().unwrap_or_default()),
                    total_double_kills: champion_stats.total_double_kills,
                    total_triple_kills: champion_stats.total_triple_kills,
                    total_quadra_kills: champion_stats.total_quadra_kills,
                    total_penta_kills: champion_stats.total_penta_kills,
                }
            })
            .collect_vec())
    }


    #[derive(FromRow)]
    struct ChampionStatsModel {
        pub champion_id: i32,
        pub total_matches: i64,
        pub total_wins: i64,
        pub avg_kda: BigDecimal,
        pub avg_kill_participation: BigDecimal,
        pub avg_kills: BigDecimal,
        pub avg_deaths: BigDecimal,
        pub avg_assists: BigDecimal,
        pub avg_gold_earned: BigDecimal,
        pub avg_cs: BigDecimal,
        pub avg_damage_dealt_to_champions: BigDecimal,
        pub avg_damage_taken: BigDecimal,
        pub total_double_kills: i64,
        pub total_triple_kills: i64,
        pub total_quadra_kills: i64,
        pub total_penta_kills: i64,
    }
}


