use crate::views::summoner_page::summoner_encounter_page::SummonerEncounterResult;
use crate::views::MatchFiltersSearch;
use leptos::prelude::*;
use leptos::server;

#[server]
pub async fn get_encounter(summoner_id: i32, filters: Option<MatchFiltersSearch>, page_number: i32, encounter_slug: String, encounter_platform: String, is_with: bool) -> Result<SummonerEncounterResult, ServerFnError> {
    let state = expect_context::<crate::ssr::AppState>();
    let db = state.db.clone();
    Ok(ssr::get_encounter_data(&db, summoner_id, filters.unwrap_or_default(), page_number, encounter_slug, encounter_platform, is_with).await?)
}


#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::backend::server_fns::get_matches::ssr::get_matches_ids;
    use crate::backend::server_fns::get_summoner::ssr::{find_summoner_by_exact_game_name_tag_line, SummonerModel};
    use crate::backend::ssr::{format_duration_since, AppResult};
    use crate::consts::champion::Champion;
    use crate::consts::platform_route::PlatformRoute;
    use crate::consts::queue::Queue;
    use crate::utils::{parse_summoner_slug, round_to_2_decimal_places};
    use crate::views::summoner_page::summoner_encounter_page::{SummonerEncounterMatch, SummonerEncounterParticipant, SummonerEncounterResult, SummonerEncounterStats};
    use crate::views::summoner_page::Summoner;
    use crate::views::MatchFiltersSearch;
    use crate::DATE_FORMAT;
    use bigdecimal::{BigDecimal, ToPrimitive};
    use chrono::{Duration, NaiveDateTime};
    use itertools::Itertools;
    use sqlx::PgPool;

    pub async fn get_encounter_data(db: &PgPool, summoner_id: i32, filters: MatchFiltersSearch, page_number: i32, encounter_slug: String, encounter_platform: String, is_with: bool) -> AppResult<SummonerEncounterResult> {
        let (encounter_game_name, encounter_tag_line) = parse_summoner_slug(encounter_slug.as_str());
        let platform_route = PlatformRoute::from(encounter_platform.as_str());
        let encounter = find_summoner_by_exact_game_name_tag_line(db, &platform_route, &encounter_game_name, &encounter_tag_line).await?;
        let summoner = find_summoner_by_id(db, summoner_id).await?;
        let match_ids = get_matches_ids(db, summoner_id, filters).await?;
        let per_page = 20;
        let offset = (page_number.max(1) - 1) * per_page;

        let matches = sqlx::query_as::<_, EncounterRowModel>(r#"
           SELECT lmp1.lol_match_id,
               lm.match_end,
               lm.platform,
               lm.queue_id,
               lm.match_duration,
               lm.match_id as riot_match_id,
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
               lmp2.summoner_id               as encounter_summoner_id,
               lmp2.won                       as encounter_won,
               lmp2.champion_id               as encounter_champion_id,
               lmp2.champ_level               as encounter_champ_level,
               lmp2.kills                     as encounter_kills,
               lmp2.deaths                    as encounter_deaths,
               lmp2.assists                   as encounter_assists,
               lmp2.kda                       as encounter_kda,
               lmp2.kill_participation        as encounter_kill_participation,
               lmp2.summoner_spell1_id        as encounter_summoner_spell1_id,
               lmp2.summoner_spell2_id        as encounter_summoner_spell2_id,
               lmp2.perk_primary_selection_id as encounter_perk_primary_selection_id,
               lmp2.perk_sub_style_id         as encounter_perk_sub_style_id,
               lmp2.item0_id                  as encounter_item0_id,
               lmp2.item1_id                  as encounter_item1_id,
               lmp2.item2_id                  as encounter_item2_id,
               lmp2.item3_id                  as encounter_item3_id,
               lmp2.item4_id                  as encounter_item4_id,
               lmp2.item5_id                  as encounter_item5_id,
               lmp2.item6_id                  as encounter_item6_id
        FROM lol_match_participants lmp1
                 JOIN lol_match_participants lmp2 ON lmp1.lol_match_id = lmp2.lol_match_id
                 join (select id, match_id, match_end, platform, queue_id, match_duration from lol_matches) lm
                      on lmp1.lol_match_id = lm.id
        WHERE lmp1.lol_match_id = ANY ($1)
          AND lmp1.summoner_id = $2
          AND lmp2.summoner_id = $3
          AND lmp1.summoner_id != lmp2.summoner_id
          AND ($4 = (lmp1.won = lmp2.won))
        ORDER BY lm.match_end DESC
        OFFSET $5 LIMIT $6
        "#)
            .bind(&match_ids)
            .bind(summoner_id)
            .bind(encounter.id)
            .bind(is_with)
            .bind(offset)
            .bind(per_page)
            .fetch_all(db)
            .await?
            .into_iter()
            .map(|row| {
                let match_duration = Duration::seconds(row.match_duration.unwrap_or_default() as i64);
                let match_duration_str = format!(
                    "{:02}:{:02}:{:02}",
                    match_duration.num_hours(),
                    match_duration.num_minutes() % 60,
                    match_duration.num_seconds() % 60
                );

                // Calculate time since match ended
                let match_ended_since = row.match_end.map_or_else(
                    || "Unknown".to_string(),
                    format_duration_since,
                );

                // Safely handle floating point operations
                let kda = (row.kda.unwrap_or_default().to_f64().unwrap_or(0.0).max(0.0) * 100.0).round() / 100.0;
                let kill_participation = (row.kill_participation.unwrap_or_default().to_f64().unwrap_or(0.0).max(0.0) * 100.0).round();
                let encounter_kda = (row.encounter_kda.unwrap_or_default().to_f64().unwrap_or(0.0).max(0.0) * 100.0).round() / 100.0;
                let encounter_kill_participation = (row.encounter_kill_participation.unwrap_or_default().to_f64().unwrap_or(0.0).max(0.0) * 100.0).round();

                SummonerEncounterMatch {
                    match_id: row.lol_match_id,
                    riot_match_id: row.riot_match_id,
                    match_ended_since,
                    match_duration: match_duration_str,
                    queue: row.queue_id.map(|q| Queue::from(q as u16).to_str()).unwrap_or_default().to_string(),
                    platform: row.platform,
                    participant: SummonerEncounterParticipant {
                        summoner_id,
                        won: row.won,
                        champion_id: row.champion_id as u16,
                        champion_name: Champion::from(row.champion_id as u16)
                            .to_str().to_string(),
                        champ_level: row.champ_level,
                        kills: row.kills,
                        deaths: row.deaths,
                        assists: row.assists,
                        kda,
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
                    },
                    encounter: SummonerEncounterParticipant {
                        summoner_id: row.encounter_summoner_id,
                        won: row.encounter_won,
                        champion_id: row.encounter_champion_id as u16,
                        champion_name: Champion::from(row.encounter_champion_id as u16)
                            .to_str().to_string(),
                        champ_level: row.encounter_champ_level,
                        kills: row.encounter_kills,
                        deaths: row.encounter_deaths,
                        assists: row.encounter_assists,
                        kda:encounter_kda,
                        kill_participation:encounter_kill_participation,
                        summoner_spell1_id: row.encounter_summoner_spell1_id.unwrap_or_default() as u16,
                        summoner_spell2_id: row.encounter_summoner_spell2_id.unwrap_or_default() as u16,
                        perk_primary_selection_id: row.encounter_perk_primary_selection_id.unwrap_or_default() as u16,
                        perk_sub_style_id: row.encounter_perk_sub_style_id.unwrap_or_default() as u16,
                        item0_id: row.encounter_item0_id.unwrap_or_default() as u32,
                        item1_id: row.encounter_item1_id.unwrap_or_default() as u32,
                        item2_id: row.encounter_item2_id.unwrap_or_default() as u32,
                        item3_id: row.encounter_item3_id.unwrap_or_default() as u32,
                        item4_id: row.encounter_item4_id.unwrap_or_default() as u32,
                        item5_id: row.encounter_item5_id.unwrap_or_default() as u32,
                        item6_id: row.encounter_item6_id.unwrap_or_default() as u32,
                    },
                }
            }).collect_vec();
        let encounter_stats = sqlx::query_as::<_, SummonerEncounterStatsModel>(r#"
            select
                count(lmp1.lol_match_id) as total_matches,
                sum(CASE WHEN lmp1.won THEN 1 ELSE 0 END) as total_wins,
                avg(lmp1.kills) as avg_kills,
                avg(lmp1.deaths) as avg_deaths,
                avg(lmp1.assists) as avg_assists,
                avg(lmp1.kda) as avg_kda,
                avg(lmp1.kill_participation) as avg_kill_participation,
                sum(CASE WHEN lmp2.won THEN 1 ELSE 0 END) as encounter_total_wins,
                avg(lmp2.kills) as encounter_avg_kills,
                avg(lmp2.deaths) as encounter_avg_deaths,
                avg(lmp2.assists) as encounter_avg_assists,
                avg(lmp2.kda) as encounter_avg_kda,
                avg(lmp2.kill_participation) as encounter_avg_kill_participation
            FROM lol_match_participants lmp1
                JOIN lol_match_participants lmp2 ON lmp1.lol_match_id = lmp2.lol_match_id
                join (select id, match_end, platform, queue_id, match_duration from lol_matches) lm
                     on lmp1.lol_match_id = lm.id
            WHERE lmp1.lol_match_id = ANY ($1)
              AND lmp1.summoner_id = $2
              AND lmp2.summoner_id = $3
              AND lmp1.summoner_id != lmp2.summoner_id
              AND ($4 = (lmp1.won = lmp2.won))
        "#)
            .bind(&match_ids)
            .bind(summoner_id)
            .bind(encounter.id)
            .bind(is_with)
            .fetch_one(db)
            .await?;
        let total_pages = (encounter_stats.total_matches as f64 / per_page as f64).ceil() as i32;
        let summoner_stats = SummonerEncounterStats {
            total_wins: encounter_stats.total_wins as i32,
            total_loses: (encounter_stats.total_matches - encounter_stats.total_wins) as i32,
            avg_kills: round_to_2_decimal_places(encounter_stats.avg_kills.to_f64().unwrap_or_default()),
            avg_deaths: round_to_2_decimal_places(encounter_stats.avg_deaths.to_f64().unwrap_or_default()),
            avg_assists: round_to_2_decimal_places(encounter_stats.avg_assists.to_f64().unwrap_or_default()),
            avg_kda: round_to_2_decimal_places(encounter_stats.avg_kda.to_f64().unwrap_or_default()),
            avg_kill_participation: round_to_2_decimal_places(encounter_stats.avg_kill_participation.to_f64().unwrap_or_default()),

        };

        let encounter_stats = SummonerEncounterStats {
            total_wins: encounter_stats.encounter_total_wins as i32,
            total_loses: (encounter_stats.total_matches - encounter_stats.encounter_total_wins) as i32,
            avg_kills: round_to_2_decimal_places(encounter_stats.encounter_avg_kills.to_f64().unwrap_or_default()),
            avg_deaths: round_to_2_decimal_places(encounter_stats.encounter_avg_deaths.to_f64().unwrap_or_default()),
            avg_assists: round_to_2_decimal_places(encounter_stats.encounter_avg_assists.to_f64().unwrap_or_default()),
            avg_kda: round_to_2_decimal_places(encounter_stats.encounter_avg_kda.to_f64().unwrap_or_default()),
            avg_kill_participation: round_to_2_decimal_places(encounter_stats.encounter_avg_kill_participation.to_f64().unwrap_or_default()),
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


    pub async fn find_summoner_by_id(db: &PgPool, summoner_id: i32) -> AppResult<Summoner> {
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
                   pp.slug            as pro_slug
            FROM summoners as ss
                     left join (select id, slug from pro_players) as pp on pp.id = ss.pro_player_id
            WHERE
                ss.id = $1
            LIMIT 1"#
        ).bind(summoner_id)
            .fetch_one(db)
            .await
            .map(|summoner_db| {
                Summoner {
                    id: summoner_db.id,
                    game_name: summoner_db.game_name,
                    tag_line: summoner_db.tag_line,
                    puuid: summoner_db.puuid,
                    platform: PlatformRoute::from(summoner_db.platform.as_str()),
                    updated_at: summoner_db.updated_at.format(DATE_FORMAT).to_string(),
                    summoner_level: summoner_db.summoner_level,
                    profile_icon_id: summoner_db.profile_icon_id as u16,
                    pro_slug: summoner_db.pro_slug,
                }
            })
            .map_err(|e| e.into())
    }


    #[derive(sqlx::FromRow)]
    pub struct SummonerEncounterStatsModel {
        pub total_matches: i64,
        pub total_wins: i64,
        pub avg_kills: BigDecimal,
        pub avg_deaths: BigDecimal,
        pub avg_assists: BigDecimal,
        pub avg_kda: BigDecimal,
        pub avg_kill_participation: BigDecimal,
        pub encounter_total_wins: i64,
        pub encounter_avg_kills: BigDecimal,
        pub encounter_avg_deaths: BigDecimal,
        pub encounter_avg_assists: BigDecimal,
        pub encounter_avg_kda: BigDecimal,
        pub encounter_avg_kill_participation: BigDecimal,
    }


    #[derive(sqlx::FromRow)]
    pub struct EncounterRowModel {
        pub lol_match_id: i32,
        pub riot_match_id: String,
        pub match_end: Option<NaiveDateTime>,
        pub platform: String,
        pub queue_id: Option<i32>,
        pub match_duration: Option<i32>,
        pub summoner_id: i32,
        pub won: bool,
        pub champion_id: i32,
        pub champ_level: i32,
        pub kills: i32,
        pub deaths: i32,
        pub assists: i32,
        pub kda: Option<BigDecimal>,
        pub kill_participation: Option<BigDecimal>,
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
        pub encounter_kda: Option<BigDecimal>,
        pub encounter_kill_participation: Option<BigDecimal>,
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