use crate::views::summoner_page::summoner_matches_page::GetSummonerMatchesResult;
use crate::views::BackEndMatchFiltersSearch;
use leptos::prelude::*;
use leptos::server_fn::codec::Rkyv;

#[server(input=Rkyv,output=Rkyv)]
pub async fn get_matches(
    summoner_id: i32,
    page_number: u16,
    filters: Option<BackEndMatchFiltersSearch>,
) -> Result<GetSummonerMatchesResult, ServerFnError> {
    let state = expect_context::<crate::ssr::AppState>();
    let db = state.db.clone();

    ssr::fetch_matches(
        &db,
        summoner_id,
        page_number as i32,
        filters.unwrap_or_default(),
    )
    .await
    .map_err(|e| e.to_server_fn_error())
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::backend::ssr::{format_duration_since, AppResult, PlatformRouteDb};
    use crate::views::summoner_page::summoner_matches_page::{
        GetSummonerMatchesResult, MatchesResultInfo, SummonerMatch, SummonerMatchParticipant,
    };
    use crate::views::BackEndMatchFiltersSearch;
    use bigdecimal::{BigDecimal, ToPrimitive};
    use chrono::NaiveDateTime;
    use common::consts::queue::Queue;
    use itertools::Itertools;

    use crate::utils::{DurationSince, ProPlayerSlug, RiotMatchId};
    use common::consts::platform_route::PlatformRoute;
    use sqlx::{FromRow, PgPool, QueryBuilder};
    use std::collections::HashMap;

    pub async fn fetch_matches(
        db: &PgPool,
        summoner_id: i32,
        page: i32,
        filters: BackEndMatchFiltersSearch,
    ) -> AppResult<GetSummonerMatchesResult> {
        let per_page = 20;
        let offset = (page.max(1) - 1) * per_page;

        let start_date = filters.start_date_to_naive();
        let end_date = filters.end_date_to_naive();

        let mut statistics_query = QueryBuilder::new(
            r#"
            SELECT
               count(*) as total_matches,
               sum(CASE WHEN lmp.won THEN 1 ELSE 0 END) as total_wins,
               avg(lmp.kills)                           as avg_kills,
               avg(lmp.deaths)                          as avg_deaths,
               avg(lmp.assists)                         as avg_assists,
               avg(lmp.kda)                             as avg_kda,
               avg(lmp.kill_participation)              as avg_kill_participation
            FROM lol_match_participants as lmp
                     left JOIN lol_matches as lm ON lm.id = lmp.lol_match_id
            WHERE lmp.summoner_id =
        "#,
        );
        let mut participants_query = QueryBuilder::new(
            r#"
            SELECT lmp.id,
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
                   lm.match_id       AS riot_match_id,
                   lm.platform       AS platform,
                   lm.queue_id       AS lol_match_queue_id,
                   lm.match_end      AS lol_match_match_end,
                   lm.match_duration AS lol_match_match_duration
            FROM lol_match_participants as lmp
                       JOIN lol_matches as lm
                                ON lm.id = lmp.lol_match_id
            WHERE lmp.summoner_id =
        "#,
        );
        statistics_query.push_bind(summoner_id);
        participants_query.push_bind(summoner_id);

        if let Some(champion_id) = filters.champion_id {
            let sql_filter = " AND lmp.champion_id = ";
            statistics_query.push(sql_filter);
            statistics_query.push_bind(champion_id as i32);
            participants_query.push(sql_filter);
            participants_query.push_bind(champion_id as i32);
        }
        if let Some(queue_id) = filters.queue_id {
            let sql_filter = " AND lm.queue_id = ";
            let queue = Queue::from(queue_id).to_u16() as i32;
            statistics_query.push(sql_filter);
            statistics_query.push_bind(queue);
            participants_query.push(sql_filter);
            participants_query.push_bind(queue);
        }

        if let Some(start_date) = start_date {
            let sql_filter = " AND lm.match_end >= ";
            statistics_query.push(sql_filter);
            statistics_query.push_bind(start_date);
            participants_query.push(sql_filter);
            participants_query.push_bind(start_date);
        }
        if let Some(end_date) = end_date {
            let sql_filter = " AND lm.match_end <= ";
            statistics_query.push(sql_filter);
            statistics_query.push_bind(end_date);
            participants_query.push(sql_filter);
            participants_query.push_bind(end_date);
        }

        participants_query.push(" ORDER BY lm.match_end DESC LIMIT 20 OFFSET ");
        participants_query.push_bind(offset);
        let (matches_statistics, matches_participants) = tokio::join!(
            statistics_query
                .build_query_as::<MatchesResultInfoModel>()
                .fetch_one(db),
            participants_query
                .build_query_as::<SummonerMatchModel>()
                .fetch_all(db),
        );
        let matches_statistics = matches_statistics?;
        let matches_participants = matches_participants?;
        let total_matches = matches_statistics.total_matches.unwrap_or_default() as u16;
        let total_pages = (total_matches as f32 / per_page as f32).ceil() as u16;

        let matches_result_info = {
            MatchesResultInfo {
                total_matches,
                total_wins: matches_statistics.total_wins.unwrap_or_default() as u16,
                avg_kills: matches_statistics
                    .avg_kills
                    .unwrap_or_default()
                    .to_f32()
                    .unwrap_or_default(),
                avg_deaths: matches_statistics
                    .avg_deaths
                    .unwrap_or_default()
                    .to_f32()
                    .unwrap_or_default(),
                avg_assists: matches_statistics
                    .avg_assists
                    .unwrap_or_default()
                    .to_f32()
                    .unwrap_or_default(),
                avg_kill_participation: (matches_statistics
                    .avg_kill_participation
                    .unwrap_or_default()
                    .to_f32()
                    .unwrap_or_default()
                    * 100.0)
                    .round() as u16,
            }
        };

        let matches_ids: Vec<_> = matches_participants
            .iter()
            .map(|row| row.lol_match_id)
            .collect();
        let mut matches = matches_participants
            .into_iter()
            .map(|row| {
                // Calculate time since match ended
                let match_ended_since = row
                    .lol_match_match_end
                    .map_or_else(|| DurationSince::new("Unknown"), format_duration_since);
                SummonerMatch {
                    summoner_id: row.summoner_id,
                    match_id: row.lol_match_id,
                    riot_match_id: RiotMatchId::new(row.riot_match_id.as_str()),
                    platform: row.platform.into(),
                    match_ended_since,
                    match_duration: row.lol_match_match_duration,
                    queue: Queue::from_u16(row.lol_match_queue_id.unwrap_or_default() as u16),
                    champion_id: row.champion_id as u16,
                    champ_level: row.champ_level as u16,
                    won: row.won,
                    kills: row.kills as u16,
                    deaths: row.deaths as u16,
                    assists: row.assists as u16,
                    kill_participation: (row.kill_participation.to_f32().unwrap_or(0.0).max(0.0)
                        * 100.0)
                        .round() as u16,
                    summoner_spell1_id: row.summoner_spell1_id.unwrap_or_default() as u16,
                    summoner_spell2_id: row.summoner_spell2_id.unwrap_or_default() as u16,
                    perk_primary_selection_id: row.perk_primary_selection_id.unwrap_or_default()
                        as u16,
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
            })
            .collect::<Vec<_>>();

        // Fetch participants for the collected match_ids

        let participants = if !matches_ids.is_empty() {
            let participant_rows = sqlx::query_as::<_, SummonerMatchParticipantModel>(
                "SELECT
                        lmp.lol_match_id,
                        lmp.summoner_id,
                        lmp.champion_id,
                        lmp.team_id
                FROM lol_match_participants as lmp
                WHERE lmp.lol_match_id = ANY($1)
                ORDER BY lmp.team_id",
            )
            .bind(&matches_ids)
            .fetch_all(db)
            .await?;
            let unique_summoner_ids = participant_rows
                .iter()
                .map(|p| p.summoner_id)
                .unique()
                .collect::<Vec<_>>();
            let summoners = get_summoner_infos_by_ids(db, unique_summoner_ids.clone()).await?;
            let encounter_counts =
                get_summoner_encounters(db, summoner_id, &unique_summoner_ids).await?;

            participant_rows
                .into_iter()
                .map(|row| {
                    let (game_name, tag_line, platform, pro_player_slug) =
                        summoners.get(&row.summoner_id).unwrap();
                    let encounter_count = *encounter_counts.get(&row.summoner_id).unwrap_or(&0);
                    SummonerMatchParticipant {
                        team_id: row.team_id as u16,
                        lol_match_id: row.lol_match_id,
                        summoner_id: row.summoner_id,
                        champion_id: row.champion_id as u16,
                        game_name: game_name.clone(),
                        tag_line: tag_line.clone(),
                        platform: *platform,
                        pro_player_slug: pro_player_slug
                            .clone()
                            .map(|pps| ProPlayerSlug::new(pps.as_str())),
                        encounter_count,
                    }
                })
                .collect_vec()
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
        Ok(GetSummonerMatchesResult {
            matches,
            total_pages,
            matches_result_info,
        })
    }

    pub async fn get_summoner_encounters(
        db: &PgPool,
        summoner_id: i32,
        encounters_ids: &[i32],
    ) -> AppResult<HashMap<i32, u16>> {
        Ok(
            sqlx::query_as::<_, (i32, i64)>(
                r#"
                 SELECT
                    lmp.summoner_id,
                    COUNT(*)    AS match_count
                 from lol_match_participants lmp
                    JOIN lol_match_participants tm ON lmp.lol_match_id = tm.lol_match_id AND tm.summoner_id = $1
                where lmp.summoner_id = ANY($2)
                group by lmp.summoner_id
            "#
            )
                .bind(summoner_id)
                .bind(encounters_ids.iter().filter(|&&id| id != summoner_id).collect::<Vec<_>>())
                .fetch_all(db)
                .await?
                .into_iter()
                .map(|(summoner_id, encounter_count)| (summoner_id, encounter_count as u16))
                .collect::<HashMap<_, _>>()
        )
    }

    pub async fn get_summoner_infos_by_ids(
        db: &PgPool,
        summoner_ids: Vec<i32>,
    ) -> AppResult<HashMap<i32, (String, String, PlatformRoute, Option<String>)>> {
        Ok(
            sqlx::query_as::<_, (i32, String, String, PlatformRouteDb, Option<String>)>(
                "SELECT
                    ss.id,
                    ss.game_name,
                    ss.tag_line,
                    ss.platform,
                    ss.pro_player_slug as pro_player_slug
            FROM summoners as ss
            WHERE ss.id = ANY($1);",
            )
            .bind(&summoner_ids)
            .fetch_all(db)
            .await?
            .into_iter()
            .map(|(id, game_name, tag_line, platform, pro_player_slug)| {
                (id, (game_name, tag_line, platform.into(), pro_player_slug))
            })
            .collect::<HashMap<_, _>>(),
        )
    }

    #[derive(FromRow)]
    pub struct MatchesResultInfoModel {
        pub total_matches: Option<i64>,
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
        pub platform: PlatformRouteDb,
        pub champion_id: i32,
        pub summoner_id: i32,
        pub summoner_spell1_id: Option<i32>,
        pub summoner_spell2_id: Option<i32>,
        #[allow(dead_code)]
        pub team_id: i32,
        pub won: bool,
        pub champ_level: i32,
        pub kill_participation: BigDecimal,
        pub kda: BigDecimal,
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
        pub champion_id: i32,
        // pub summoner_name: String,
        // pub summoner_tag_line: String,
        // pub summoner_platform: String,
        // pub pro_player_slug:Option<String>,
    }
}
