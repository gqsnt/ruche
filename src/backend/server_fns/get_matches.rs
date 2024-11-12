use crate::views::summoner_page::summoner_matches_page::GetSummonerMatchesResult;
use crate::views::MatchFiltersSearch;
use leptos::prelude::*;
use leptos::server;

#[server]
pub async fn get_matches(summoner_id: i32, page_number: i32, filters: Option<MatchFiltersSearch>) -> Result<GetSummonerMatchesResult, ServerFnError> {
    let state = expect_context::<crate::ssr::AppState>();
    let db = state.db.clone();

    ssr::inner_get_matches(&db, summoner_id, page_number, filters.unwrap_or_default()).await.map_err(|e| e.to_server_fn_error())
}


#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::views::summoner_page::summoner_matches_page::{GetSummonerMatchesResult, MatchesResultInfo, SummonerMatch, SummonerMatchParticipant};
    use crate::views::MatchFiltersSearch;
    use bigdecimal::{BigDecimal, ToPrimitive};
    use chrono::{Duration, NaiveDateTime};
    use itertools::Itertools;

    use crate::backend::ssr::{format_duration_since, parse_date, AppResult};
    use crate::consts::queue::Queue;
    use sqlx::{FromRow, PgPool, QueryBuilder};

    pub async fn inner_get_matches(
        db: &PgPool,
        summoner_id: i32,
        page: i32,
        filters: MatchFiltersSearch,
    ) -> AppResult<GetSummonerMatchesResult> {
        let start_date = parse_date(filters.start_date.clone());
        let end_date = parse_date(filters.end_date.clone());
        let per_page = 20;
        let offset = (page.max(1) - 1) * per_page;
        let mut aggregate_query = QueryBuilder::new(r#"
            SELECT
                count(lmp.lol_match_id) as total_count,
                sum(CASE WHEN lmp.won THEN 1 ELSE 0 END) as total_wins,
                avg(lmp.kills)  as avg_kills,
                avg(lmp.deaths)  as avg_deaths,
                avg(lmp.assists)  as avg_assists,
                avg(lmp.kda)  as avg_kda,
                avg(lmp.kill_participation) as avg_kill_participation
            FROM lol_match_participants as lmp
            INNER JOIN (SELECT id, queue_id, match_end FROM lol_matches) as lm ON lm.id = lmp.lol_match_id
            WHERE
                lmp.summoner_id =
        "#);
        aggregate_query.push_bind(summoner_id);

        let mut participant_query = QueryBuilder::new(r#"
            SELECT
                lmp.id,
                lmp.lol_match_id,
                lmp.champion_id,
                lmp.summoner_id,
                lmp.team_id,
                lmp.won,
                lmp.champ_level,
                lmp.kill_participation,
                lmp.kda,
                lmp.kills,
                lmp.deaths,
                lmp.assists,
                lmp.summoner_spell1_id,
                lmp.summoner_spell2_id,
                lmp.perk_primary_selection_id,
                lmp.perk_sub_style_id,
                lmp.item0_id,
                lmp.item1_id,
                lmp.item2_id,
                lmp.item3_id,
                lmp.item4_id,
                lmp.item5_id,
                lmp.item6_id,
                lm.match_id AS riot_match_id,
                lm.platform AS platform,
                lm.queue_id AS lol_match_queue_id,
                lm.match_end AS lol_match_match_end,
                lm.match_duration AS lol_match_match_duration
            FROM lol_match_participants as lmp
            INNER JOIN (SELECT id,match_id,platform, queue_id,match_duration, match_end FROM lol_matches) as lm ON lm.id = lmp.lol_match_id
            WHERE
                lmp.summoner_id =
        "#);
        participant_query.push_bind(summoner_id);


        if let Some(champion_id) = filters.champion_id {
            let sql_filter = " AND lmp.champion_id = ";
            participant_query.push(&sql_filter);
            participant_query.push_bind(champion_id);
            aggregate_query.push(&sql_filter);
            aggregate_query.push_bind(champion_id);
        }
        if let Some(queue_id) = filters.queue_id {
            let sql_filter = " AND lm.queue_id = ";
            participant_query.push(&sql_filter);
            participant_query.push_bind(queue_id);
            aggregate_query.push(&sql_filter);
            aggregate_query.push_bind(queue_id);
        }

        if let Some(start_date) = start_date {
            let sql_filter = " AND lm.match_end >= ";
            participant_query.push(&sql_filter);
            participant_query.push_bind(start_date);
            aggregate_query.push(&sql_filter);
            aggregate_query.push_bind(start_date);
        }
        if let Some(end_date) = end_date {
            let sql_filter = " AND lm.match_end <= ";
            participant_query.push(&sql_filter);
            participant_query.push_bind(end_date);
            aggregate_query.push(&sql_filter);
            aggregate_query.push_bind(end_date);
        }

        participant_query.push(" ORDER BY lm.match_end DESC LIMIT ");
        participant_query.push_bind(per_page);
        participant_query.push(" OFFSET ");
        participant_query.push_bind(offset);
        let ag_build = aggregate_query.build_query_as::<MatchesResultInfoModel>();

        let aggregate_result = ag_build
            .fetch_one(db)
            .await?;
        let results = participant_query
            .build_query_as::<SummonerMatchModel>()
            .fetch_all(db)
            .await?;


        let matches_result_info = {
            let total_matches = aggregate_result.total_count.unwrap_or_default() as i32;
            let total_wins = aggregate_result.total_wins.unwrap_or_default() as i32;
            let total_losses = total_matches - total_wins;
            let round_2 = |x: f64| (x * 100.0).round() / 100.0;
            MatchesResultInfo {
                total_matches,
                total_wins,
                total_losses,
                avg_kills: round_2(aggregate_result.avg_kills.clone().unwrap_or_default().to_f64().unwrap_or_default()),
                avg_deaths: round_2(aggregate_result.avg_deaths.clone().unwrap_or_default().to_f64().unwrap_or_default()),
                avg_assists: round_2(aggregate_result.avg_assists.clone().unwrap_or_default().to_f64().unwrap_or_default()),
                avg_kda: round_2(aggregate_result.avg_kda.clone().unwrap_or_default().to_f64().unwrap_or_default()),
                avg_kill_participation: (aggregate_result.avg_kill_participation.clone().unwrap_or_default().to_f64().unwrap_or_default() * 100.0) as i32,
            }
        };

        let matches_ids: Vec<_> = results.iter().map(|row| row.lol_match_id).collect();
        let mut matches = results.into_iter().map(|row| {
            let match_duration = Duration::seconds(row.lol_match_match_duration.unwrap_or_default() as i64);
            let match_duration_str = format!(
                "{:02}:{:02}:{:02}",
                match_duration.num_hours(),
                match_duration.num_minutes() % 60,
                match_duration.num_seconds() % 60
            );

            // Calculate time since match ended
            let match_ended_since = row.lol_match_match_end.map_or_else(
                || "Unknown".to_string(),
                |dt| format_duration_since(dt),
            );

            // Safely handle floating point operations
            let kda = (row.kda.unwrap_or_default().to_f64().unwrap_or(0.0).max(0.0) * 100.0).round() / 100.0;
            let kill_participation = (row.kill_participation.unwrap_or_default().to_f64().unwrap_or(0.0).max(0.0) * 100.0).round();
            SummonerMatch {
                summoner_id: row.summoner_id,
                match_id: row.lol_match_id,
                riot_match_id: row.riot_match_id,
                platform: row.platform.unwrap_or_default(),
                match_ended_since,
                match_duration: match_duration_str,
                queue: row.lol_match_queue_id.map(|q| Queue::from(q as u16).to_str()).unwrap_or_default().to_string(),
                champion_id: row.champion_id as u16,
                champ_level: row.champ_level,
                won: row.won,
                kda,
                kills: row.kills,
                deaths: row.deaths,
                assists: row.assists,
                kill_participation,
                summoner_spell1_id: row.summoner_spell1_id.unwrap_or_default() as u16,
                summoner_spell2_id: row.summoner_spell2_id.unwrap_or_default() as u16,
                perk_primary_selection_id: row.perk_primary_selection_id.unwrap_or_default() as u16,
                perk_sub_style_id: row.perk_sub_style_id.unwrap_or_default() as u16,
                item0_id: row.item0_id.unwrap_or_default() as u32,
                item1_id: row.item1_id.unwrap_or_default() as u32,
                item2_id: row.item2_id.unwrap_or_default() as u32,
                item3_id: row.item3_id.unwrap_or_default() as u32,
                item4_id: row.item4_id.unwrap_or_default() as u32,
                item5_id: row.item5_id.unwrap_or_default() as u32,
                item6_id: row.item6_id.unwrap_or_default() as u32,
                participants: vec![],
            }
        }).collect::<Vec<_>>();
        let total_pages = (matches_result_info.total_matches as f64 / per_page as f64).ceil() as i32;

        // Fetch participants for the collected match_ids

        let participants = if !matches_ids.is_empty() {
            let participant_rows = sqlx::query_as::<_, SummonerMatchParticipantModel>(
                "SELECT
                    lol_match_participants.lol_match_id,
                    lol_match_participants.summoner_id,
                    lol_match_participants.champion_id,
                    lol_match_participants.team_id,
                    summoners.game_name AS summoner_name,
                    summoners.tag_line AS summoner_tag_line,
                    summoners.platform AS summoner_platform
                FROM lol_match_participants
                INNER JOIN summoners ON summoners.id = lol_match_participants.summoner_id
                WHERE lol_match_participants.lol_match_id = ANY($1)
                ORDER BY lol_match_participants.team_id ASC;"
            )
                .bind(&matches_ids)
                .fetch_all(db)
                .await?;
            participant_rows.into_iter()
                .map(|row| SummonerMatchParticipant {
                    team_id: row.team_id,
                    lol_match_id: row.lol_match_id,
                    summoner_id: row.summoner_id,
                    summoner_name: row.summoner_name,
                    champion_id: row.champion_id as u16,
                    summoner_tag_line: row.summoner_tag_line,
                    summoner_platform: row.summoner_platform,
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        // Group participants by match_id
        let participants_grouped = participants
            .into_iter()
            .into_group_map_by(|p| p.lol_match_id);

        // Assign participants to their respective matches
        for match_ in matches.iter_mut() {
            if let Some(participants) = participants_grouped.get(&match_.match_id) {
                match_.participants = participants.clone();
            }
        }

        Ok(GetSummonerMatchesResult { matches, total_pages: total_pages as i64, matches_result_info })
    }


    #[derive(FromRow)]
    pub struct MatchesResultInfoModel {
        #[allow(dead_code)]
        pub total_count: Option<i64>,
        pub total_wins: Option<i64>,
        pub avg_kills: Option<BigDecimal>,
        pub avg_deaths: Option<BigDecimal>,
        pub avg_assists: Option<BigDecimal>,
        pub avg_kda: Option<BigDecimal>,
        pub avg_kill_participation: Option<BigDecimal>,
    }

    #[derive(FromRow)]
    pub struct SummonerMatchModel {
        #[allow(dead_code)]
        pub id: i32,
        pub lol_match_id: i32,
        pub riot_match_id: String,
        pub platform: Option<String>,
        pub champion_id: i32,
        pub summoner_id: i32,
        pub summoner_spell1_id: Option<i32>,
        pub summoner_spell2_id: Option<i32>,
        #[allow(dead_code)]
        pub team_id: i32,
        pub won: bool,
        pub champ_level: i32,
        pub kill_participation: Option<BigDecimal>,
        pub kda: Option<BigDecimal>,
        pub kills: i32,
        pub deaths: i32,
        pub assists: i32,
        pub perk_primary_selection_id: Option<i32>,
        pub perk_sub_style_id: Option<i32>,
        pub item0_id: Option<i64>,
        pub item1_id: Option<i64>,
        pub item2_id: Option<i64>,
        pub item3_id: Option<i64>,
        pub item4_id: Option<i64>,
        pub item5_id: Option<i64>,
        pub item6_id: Option<i64>,
        pub lol_match_queue_id: Option<i32>,
        pub lol_match_match_end: Option<NaiveDateTime>,
        pub lol_match_match_duration: Option<i32>,
    }


    #[derive(FromRow)]
    pub struct SummonerMatchParticipantModel {
        pub team_id: i32,
        pub lol_match_id: i32,
        pub summoner_id: i32,
        pub summoner_name: String,
        pub champion_id: i32,
        pub summoner_tag_line: String,
        pub summoner_platform: String,
    }
}


