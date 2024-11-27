use crate::consts::platform_route::PlatformRoute;
use crate::utils::SummonerSlug;
use crate::views::summoner_page::summoner_encounter_page::SummonerEncounterResult;
use crate::views::BackEndMatchFiltersSearch;
use leptos::prelude::*;
use leptos::server;
use leptos::server_fn::codec::Rkyv;

#[server(input=Rkyv, output=Rkyv)]
pub async fn get_encounter(
    summoner_id: i32,
    page_number: u16,
    is_with: bool,
    encounter_platform: PlatformRoute,
    encounter_slug: SummonerSlug,
    filters: Option<BackEndMatchFiltersSearch>,
) -> Result<SummonerEncounterResult, ServerFnError> {
    let state = expect_context::<crate::ssr::AppState>();
    let db = state.db.clone();

    Ok(ssr::get_encounter_data(
        &db,
        summoner_id,
        filters.unwrap_or_default(),
        page_number as i32,
        encounter_slug.to_string(),
        encounter_platform,
        is_with,
    )
    .await?)
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::backend::server_fns::get_summoner::ssr::{
        find_summoner_by_exact_game_name_tag_line, SummonerModel,
    };
    use crate::backend::ssr::{format_duration_since, AppError, AppResult, PlatformRouteDb};
    use crate::consts::platform_route::PlatformRoute;
    use crate::consts::queue::Queue;
    use crate::utils::{
        parse_summoner_slug, DurationSince, GameName, ProPlayerSlug, RiotMatchId, TagLine,
    };
    use crate::views::summoner_page::summoner_encounter_page::{
        SummonerEncounterMatch, SummonerEncounterParticipant, SummonerEncounterResult,
        SummonerEncounterStats,
    };
    use crate::views::summoner_page::Summoner;
    use crate::views::BackEndMatchFiltersSearch;
    use bigdecimal::{BigDecimal, ToPrimitive};
    use chrono::NaiveDateTime;
    use itertools::Itertools;
    use sqlx::{PgPool, QueryBuilder};

    pub async fn get_encounter_data(
        db: &PgPool,
        summoner_id: i32,
        filters: BackEndMatchFiltersSearch,
        page_number: i32,
        encounter_slug: String,
        encounter_platform: PlatformRoute,
        is_with: bool,
    ) -> AppResult<SummonerEncounterResult> {
        let (encounter_game_name, encounter_tag_line) =
            parse_summoner_slug(encounter_slug.as_str());

        let summoner = find_summoner_by_id(db, summoner_id).await?;
        let encounter = find_summoner_by_exact_game_name_tag_line(
            db,
            encounter_platform,
            encounter_game_name,
            encounter_tag_line,
        )
        .await?
        .ok_or(AppError::NotFound)?;
        let per_page = 20;
        let offset = (page_number.max(1) - 1) * per_page;

        let start_date = filters.start_date_to_naive();
        let end_date = filters.end_date_to_naive();
        let mut stats_query = QueryBuilder::new(
            r#"
            select
                count(lmp1.lol_match_id) as total_matches,
                sum(CASE WHEN lmp1.won THEN 1 ELSE 0 END) as total_wins,
                avg(lmp1.kills) as avg_kills,
                avg(lmp1.deaths) as avg_deaths,
                avg(lmp1.assists) as avg_assists,
                avg(lmp1.kill_participation) as avg_kill_participation,
                sum(CASE WHEN lmp2.won THEN 1 ELSE 0 END) as encounter_total_wins,
                avg(lmp2.kills) as encounter_avg_kills,
                avg(lmp2.deaths) as encounter_avg_deaths,
                avg(lmp2.assists) as encounter_avg_assists,
                avg(lmp2.kill_participation) as encounter_avg_kill_participation
            FROM lol_match_participants lmp1
                     left JOIN lol_matches lm ON lm.id = lmp1.lol_match_id
                     JOIN lol_match_participants lmp2 ON lmp2.lol_match_id = lmp1.lol_match_id and lmp2.summoner_id =
        "#,
        );

        let mut query = QueryBuilder::new(
            r#"
            SELECT
               lmp1.lol_match_id,
               lm.match_end,
               lm.platform,
               lm.queue_id,
               lm.match_duration,
               lm.match_id  AS riot_match_id,
               lmp1.summoner_id,
               lmp1.won,
               lmp1.champion_id,
               lmp1.champ_level,
               lmp1.kills,
               lmp1.deaths,
               lmp1.assists,
               lmp1.kda,
               lmp1.kill_participation,
               lmp1.summoner_spell1_id,
               lmp1.summoner_spell2_id,
               lmp1.perk_primary_selection_id,
               lmp1.perk_sub_style_id,
               lmp1.item0_id,
               lmp1.item1_id,
               lmp1.item2_id,
               lmp1.item3_id,
               lmp1.item4_id,
               lmp1.item5_id,
               lmp1.item6_id,
               lmp2.summoner_id               AS encounter_summoner_id,
               lmp2.won                       AS encounter_won,
               lmp2.champion_id               AS encounter_champion_id,
               lmp2.champ_level               AS encounter_champ_level,
               lmp2.kills                     AS encounter_kills,
               lmp2.deaths                    AS encounter_deaths,
               lmp2.assists                   AS encounter_assists,
               lmp2.kda                       AS encounter_kda,
               lmp2.kill_participation        AS encounter_kill_participation,
               lmp2.summoner_spell1_id        AS encounter_summoner_spell1_id,
               lmp2.summoner_spell2_id        AS encounter_summoner_spell2_id,
               lmp2.perk_primary_selection_id AS encounter_perk_primary_selection_id,
               lmp2.perk_sub_style_id         AS encounter_perk_sub_style_id,
               lmp2.item0_id                  AS encounter_item0_id,
               lmp2.item1_id                  AS encounter_item1_id,
               lmp2.item2_id                  AS encounter_item2_id,
               lmp2.item3_id                  AS encounter_item3_id,
               lmp2.item4_id                  AS encounter_item4_id,
               lmp2.item5_id                  AS encounter_item5_id,
               lmp2.item6_id                  AS encounter_item6_id
           FROM lol_match_participants lmp1
                     left JOIN lol_matches lm ON lm.id = lmp1.lol_match_id
                     JOIN lol_match_participants lmp2 ON lmp2.lol_match_id = lmp1.lol_match_id and lmp2.summoner_id =
        "#,
        );
        query.push_bind(encounter.id);
        stats_query.push_bind(encounter.id);
        query.push(" AND ");
        stats_query.push(" AND ");
        query.push_bind(is_with);
        stats_query.push_bind(is_with);
        query.push(" = (lmp2.won = lmp1.won) where lmp1.summoner_id = ");
        stats_query.push(" = (lmp2.won = lmp1.won) where lmp1.summoner_id = ");
        query.push_bind(summoner_id);
        stats_query.push_bind(summoner_id);

        if let Some(champion_id) = filters.champion_id {
            let sql_filter = " AND lmp1.champion_id = ";
            query.push(sql_filter);
            query.push_bind(champion_id as i32);
            stats_query.push(sql_filter);
            stats_query.push_bind(champion_id as i32);
        }

        if let Some(queue_id) = filters.queue_id {
            let sql_filter = " AND lm.queue_id = ";
            query.push(sql_filter);
            query.push_bind(Queue::from(queue_id).to_u16() as i32);
            stats_query.push(sql_filter);
            stats_query.push_bind(Queue::from(queue_id).to_u16() as i32);
        }

        if let Some(start_date) = start_date {
            let sql_filter = " AND lm.match_end >= ";
            query.push(sql_filter);
            query.push_bind(start_date);
            stats_query.push(sql_filter);
            stats_query.push_bind(start_date);
        }

        if let Some(end_date) = end_date {
            let sql_filter = " AND lm.match_end <= ";
            query.push(sql_filter);
            query.push_bind(end_date);
            stats_query.push(sql_filter);
            stats_query.push_bind(end_date);
        }

        query.push(" order by lm.match_end desc limit 20 offset ");
        query.push_bind(offset);
        let (encounter_stats, matches) = tokio::join!(
            stats_query
                .build_query_as::<SummonerEncounterStatsModel>()
                .fetch_one(db),
            query.build_query_as::<EncounterRowModel>().fetch_all(db)
        );
        let encounter_stats = encounter_stats?;
        let matches = matches?;

        let matches = matches
            .into_iter()
            .map(|row| {
                // Calculate time since match ended
                let match_ended_since = row
                    .match_end
                    .map_or_else(|| DurationSince::new("Unknown"), format_duration_since);

                SummonerEncounterMatch {
                    match_id: row.lol_match_id,
                    riot_match_id: RiotMatchId::new(row.riot_match_id.as_str()),
                    match_ended_since,
                    match_duration: row.match_duration,
                    queue: row.queue_id.map(|q| Queue::from_u16(q as u16)).unwrap(),
                    platform: row.platform.into(),
                    participant: SummonerEncounterParticipant {
                        summoner_id,
                        won: row.won,
                        champion_id: row.champion_id as u16,
                        champ_level: row.champ_level as u16,
                        kills: row.kills as u16,
                        deaths: row.deaths as u16,
                        assists: row.assists as u16,
                        kill_participation: (row
                            .kill_participation
                            .to_f32()
                            .unwrap_or(0.0)
                            .max(0.0)
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
                    },
                    encounter: SummonerEncounterParticipant {
                        summoner_id: row.encounter_summoner_id,
                        won: row.encounter_won,
                        champion_id: row.encounter_champion_id as u16,
                        champ_level: row.encounter_champ_level as u16,
                        kills: row.encounter_kills as u16,
                        deaths: row.encounter_deaths as u16,
                        assists: row.encounter_assists as u16,
                        kill_participation: (row
                            .encounter_kill_participation
                            .to_f32()
                            .unwrap_or(0.0)
                            .max(0.0)
                            * 100.0)
                            .round() as u16,
                        summoner_spell1_id: row.encounter_summoner_spell1_id.unwrap_or_default()
                            as u16,
                        summoner_spell2_id: row.encounter_summoner_spell2_id.unwrap_or_default()
                            as u16,
                        perk_primary_selection_id: row
                            .encounter_perk_primary_selection_id
                            .unwrap_or_default()
                            as u16,
                        perk_sub_style_id: row.encounter_perk_sub_style_id.unwrap_or_default()
                            as u16,
                        item0_id: row.encounter_item0_id.unwrap_or_default() as u32,
                        item1_id: row.encounter_item1_id.unwrap_or_default() as u32,
                        item2_id: row.encounter_item2_id.unwrap_or_default() as u32,
                        item3_id: row.encounter_item3_id.unwrap_or_default() as u32,
                        item4_id: row.encounter_item4_id.unwrap_or_default() as u32,
                        item5_id: row.encounter_item5_id.unwrap_or_default() as u32,
                        item6_id: row.encounter_item6_id.unwrap_or_default() as u32,
                    },
                }
            })
            .collect_vec();

        let total_pages = (encounter_stats.total_matches.unwrap_or_default() as f64
            / per_page as f64)
            .ceil() as u16;
        let summoner_stats = SummonerEncounterStats {
            total_wins: encounter_stats.total_wins.unwrap_or_default() as u16,
            total_matches: encounter_stats.total_matches.unwrap_or_default() as u16,
            avg_kills: encounter_stats
                .avg_kills
                .unwrap_or_default()
                .to_f32()
                .unwrap_or_default(),
            avg_deaths: encounter_stats
                .avg_deaths
                .unwrap_or_default()
                .to_f32()
                .unwrap_or_default(),
            avg_assists: encounter_stats
                .avg_assists
                .unwrap_or_default()
                .to_f32()
                .unwrap_or_default(),
            avg_kill_participation: (encounter_stats
                .avg_kill_participation
                .unwrap_or_default()
                .to_f32()
                .unwrap_or_default()
                * 100.0)
                .round() as u16,
        };

        let encounter_stats = SummonerEncounterStats {
            total_matches: encounter_stats.total_matches.unwrap_or_default() as u16,
            total_wins: encounter_stats.encounter_total_wins.unwrap_or_default() as u16,
            avg_kills: encounter_stats
                .encounter_avg_kills
                .unwrap_or_default()
                .to_f32()
                .unwrap_or_default(),
            avg_deaths: encounter_stats
                .encounter_avg_deaths
                .unwrap_or_default()
                .to_f32()
                .unwrap_or_default(),
            avg_assists: encounter_stats
                .encounter_avg_assists
                .unwrap_or_default()
                .to_f32()
                .unwrap_or_default(),
            avg_kill_participation: (encounter_stats
                .encounter_avg_kill_participation
                .unwrap_or_default()
                .to_f32()
                .unwrap_or_default()
                * 100.0)
                .round() as u16,
        };
        Ok(SummonerEncounterResult {
            total_pages,
            matches,
            summoner_stats,
            encounter_stats,
            summoner,
            encounter,
        })
    }

    pub async fn find_summoner_model_by_id(
        db: &PgPool,
        summoner_id: i32,
    ) -> AppResult<SummonerModel> {
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
                   ss.pro_player_slug            as pro_slug
            FROM summoners as ss
            WHERE
                ss.id = $1"#,
        )
        .bind(summoner_id)
        .fetch_one(db)
        .await
        .map_err(|e| e.into())
    }

    pub async fn find_summoner_puuid_by_id(db: &PgPool, summoner_id: i32) -> AppResult<String> {
        sqlx::query_scalar::<_, String>(
            r#"
            SELECT
                   ss.puuid
            FROM summoners as ss
            WHERE
                ss.id = $1"#,
        )
        .bind(summoner_id)
        .fetch_one(db)
        .await
        .map_err(|e| e.into())
    }

    pub async fn find_summoner_by_id(db: &PgPool, summoner_id: i32) -> AppResult<Summoner> {
        find_summoner_model_by_id(db, summoner_id)
            .await
            .map(|summoner_db| Summoner {
                id: summoner_db.id,
                game_name: GameName::new(summoner_db.game_name.as_str()),
                tag_line: TagLine::new(summoner_db.tag_line.as_str()),
                platform: PlatformRoute::from(summoner_db.platform),
                summoner_level: summoner_db.summoner_level as u16,
                profile_icon_id: summoner_db.profile_icon_id as u16,
                pro_slug: summoner_db
                    .pro_slug
                    .map(|slug| ProPlayerSlug::new(slug.as_str())),
            })
    }

    #[derive(sqlx::FromRow)]
    pub struct SummonerEncounterStatsModel {
        pub total_matches: Option<i64>,
        pub total_wins: Option<i64>,
        pub avg_kills: Option<BigDecimal>,
        pub avg_deaths: Option<BigDecimal>,
        pub avg_assists: Option<BigDecimal>,
        pub avg_kill_participation: Option<BigDecimal>,
        pub encounter_total_wins: Option<i64>,
        pub encounter_avg_kills: Option<BigDecimal>,
        pub encounter_avg_deaths: Option<BigDecimal>,
        pub encounter_avg_assists: Option<BigDecimal>,
        pub encounter_avg_kill_participation: Option<BigDecimal>,
    }

    #[derive(sqlx::FromRow)]
    pub struct EncounterRowModel {
        pub lol_match_id: i32,
        pub riot_match_id: String,
        pub match_end: Option<NaiveDateTime>,
        pub platform: PlatformRouteDb,
        pub queue_id: Option<i32>,
        pub match_duration: Option<i32>,
        pub summoner_id: i32,
        pub won: bool,
        pub champion_id: i32,
        pub champ_level: i32,
        pub kills: i32,
        pub deaths: i32,
        pub assists: i32,
        pub kill_participation: BigDecimal,
        pub summoner_spell1_id: Option<i32>,
        pub summoner_spell2_id: Option<i32>,
        pub perk_primary_selection_id: Option<i32>,
        pub perk_sub_style_id: Option<i32>,
        pub item0_id: Option<i64>,
        pub item1_id: Option<i64>,
        pub item2_id: Option<i64>,
        pub item3_id: Option<i64>,
        pub item4_id: Option<i64>,
        pub item5_id: Option<i64>,
        pub item6_id: Option<i64>,
        pub encounter_summoner_id: i32,
        pub encounter_won: bool,
        pub encounter_champion_id: i32,
        pub encounter_champ_level: i32,
        pub encounter_kills: i32,
        pub encounter_deaths: i32,
        pub encounter_assists: i32,
        pub encounter_kill_participation: BigDecimal,
        pub encounter_summoner_spell1_id: Option<i32>,
        pub encounter_summoner_spell2_id: Option<i32>,
        pub encounter_perk_primary_selection_id: Option<i32>,
        pub encounter_perk_sub_style_id: Option<i32>,
        pub encounter_item0_id: Option<i64>,
        pub encounter_item1_id: Option<i64>,
        pub encounter_item2_id: Option<i64>,
        pub encounter_item3_id: Option<i64>,
        pub encounter_item4_id: Option<i64>,
        pub encounter_item5_id: Option<i64>,
        pub encounter_item6_id: Option<i64>,
    }
}
