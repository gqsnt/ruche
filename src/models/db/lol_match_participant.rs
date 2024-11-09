use std::collections::HashMap;
use crate::apis::MatchFiltersSearch;
use crate::views::summoner_page::summoner_matches_page::{GetSummonerMatchesResult, MatchesResultInfo};
use crate::consts::Queue;
use crate::error_template::{AppError, AppResult};
use crate::models::db::db_model::{LolMatchParticipantDetailsQueryResult, LolMatchParticipantLiveGameDb, LolMatchParticipantMatchesChildQueryResult, LolMatchParticipantMatchesQueryAggregateResult, LolMatchParticipantMatchesQueryResult, LolSummonerChampionResult};
use crate::models::entities::lol_match_participant::{LolMatchDefaultParticipantMatchesPage, LolMatchParticipant, LolMatchParticipantMatchesDetailPage, LolMatchParticipantMatchesPage, LolSummonerChampionPage};
use crate::models::update::summoner_matches::TempParticipant;
use bigdecimal::ToPrimitive;
use chrono::{Duration, NaiveDateTime, Utc};
use itertools::Itertools;
use leptos::prelude::BindAttribute;
use sqlx::{PgPool, QueryBuilder, Row};
use unzip_n::unzip_n;
use crate::models::db::{parse_date, round_to_2_decimal_places};

unzip_n!(23);
unzip_n!(11);
unzip_n!(7);



impl LolMatchParticipant {
    pub async fn get_details(db: &PgPool, match_id: i32) -> AppResult<Vec<LolMatchParticipantMatchesDetailPage>> {
        Ok(sqlx::query_as::<_, LolMatchParticipantDetailsQueryResult>(
            "SELECT
                lmp.id,
                lmp.lol_match_id,
                lmp.summoner_id,
                ss.game_name AS summoner_name,
                ss.tag_line AS summoner_tag_line,
                ss.platform AS summoner_platform,
                ss.profile_icon_id AS summoner_icon_id,
                ss.summoner_level AS summoner_level,
                lmp.champion_id,
                lmp.team_id,
                lmp.won,
                lmp.kills,
                lmp.deaths,
                lmp.assists,
                lmp.champ_level,
                lmp.kda,
                lmp.kill_participation,
                lmp.damage_dealt_to_champions,
                lmp.damage_taken,
                lmp.gold_earned,
                lmp.wards_placed,
                lmp.cs,
                lmp.summoner_spell1_id,
                lmp.summoner_spell2_id,
                lmp.perk_defense_id,
                lmp.perk_flex_id,
                lmp.perk_offense_id,
                lmp.perk_primary_style_id,
                lmp.perk_sub_style_id,
                lmp.perk_primary_selection_id,
                lmp.perk_primary_selection1_id,
                lmp.perk_primary_selection2_id,
                lmp.perk_primary_selection3_id,
                lmp.perk_sub_selection1_id,
                lmp.perk_sub_selection2_id,
                lmp.item0_id,
                lmp.item1_id,
                lmp.item2_id,
                lmp.item3_id,
                lmp.item4_id,
                lmp.item5_id,
                lmp.item6_id
            FROM lol_match_participants  as lmp
            INNER JOIN summoners as ss ON ss.id = lmp.summoner_id
            WHERE lmp.lol_match_id = $1;",
        )
            .bind(match_id)
            .fetch_all(db)
            .await
            .map_err(|_| AppError::CustomError("Error fetching match details".to_string()))?
            .into_iter().map(|lmp| {
            LolMatchParticipantMatchesDetailPage {
                id: lmp.id,
                lol_match_id: lmp.lol_match_id,
                summoner_id: lmp.summoner_id,
                summoner_name: lmp.summoner_name,
                summoner_tag_line: lmp.summoner_tag_line,
                summoner_platform: lmp.summoner_platform,
                summoner_icon_id: lmp.summoner_icon_id,
                summoner_level: lmp.summoner_level,
                champion_id: lmp.champion_id,
                team_id: lmp.team_id,
                won: lmp.won,
                kills: lmp.kills,
                deaths: lmp.deaths,
                assists: lmp.assists,
                champ_level: lmp.champ_level,
                kda: lmp.kda.map_or(0.0, |bd| bd.to_f64().unwrap_or(0.0)),
                kill_participation: lmp.kill_participation.map_or(0.0, |bd| bd.to_f64().unwrap_or(0.0)),
                damage_dealt_to_champions: lmp.damage_dealt_to_champions,
                damage_taken: lmp.damage_taken,
                gold_earned: lmp.gold_earned,
                wards_placed: lmp.wards_placed,
                cs: lmp.cs,
                summoner_spell1_id: lmp.summoner_spell1_id.unwrap_or_default(),
                summoner_spell2_id: lmp.summoner_spell2_id.unwrap_or_default(),
                perk_defense_id: lmp.perk_defense_id.unwrap_or_default(),
                perk_flex_id: lmp.perk_flex_id.unwrap_or_default(),
                perk_offense_id: lmp.perk_offense_id.unwrap_or_default(),
                perk_primary_style_id: lmp.perk_primary_style_id.unwrap_or_default(),
                perk_sub_style_id: lmp.perk_sub_style_id.unwrap_or_default(),
                perk_primary_selection_id: lmp.perk_primary_selection_id.unwrap_or_default(),
                perk_primary_selection1_id: lmp.perk_primary_selection1_id.unwrap_or_default(),
                perk_primary_selection2_id: lmp.perk_primary_selection2_id.unwrap_or_default(),
                perk_primary_selection3_id: lmp.perk_primary_selection3_id.unwrap_or_default(),
                perk_sub_selection1_id: lmp.perk_sub_selection1_id.unwrap_or_default(),
                perk_sub_selection2_id: lmp.perk_sub_selection2_id.unwrap_or_default(),
                item0_id: lmp.item0_id.unwrap_or_default(),
                item1_id: lmp.item1_id.unwrap_or_default(),
                item2_id: lmp.item2_id.unwrap_or_default(),
                item3_id: lmp.item3_id.unwrap_or_default(),
                item4_id: lmp.item4_id.unwrap_or_default(),
                item5_id: lmp.item5_id.unwrap_or_default(),
                item6_id: lmp.item6_id.unwrap_or_default(),
                items_event_timeline: Vec::new(),
                skills_timeline: vec![],
            }
        }).collect::<Vec<_>>())
    }

    pub async fn get_live_game_stats(
        db:&PgPool,
        summoner_ids: &[i32],
    )->AppResult<HashMap<i32, HashMap<i32, LolMatchParticipantLiveGameDb>>>{
        let query_results = sqlx::query_as::<_,LolMatchParticipantLiveGameDb>(r#"
            select
                summoner_id,
                champion_id,
                count(lmp.lol_match_id) as total_match,
                sum(CASE WHEN won THEN 1 ELSE 0 END) as total_win,
                avg(lmp.kills) as avg_kills,
                avg(lmp.deaths) as avg_deaths,
                avg(lmp.assists) as avg_assists
            from lol_match_participants as lmp
                inner join (select id, queue_id,match_end  from lol_matches) as lm on lmp.lol_match_id = lm.id
            where lmp.summoner_id = ANY($1) and lm.queue_id = 420 and lm.match_end >= '2024-09-25 12:00:00'
            group by lmp.summoner_id, lmp.champion_id;
        "#)  // 420 is the queue id for ranked solo/duo and 2024-09-25 is the split 3 s14 start date
            .bind(summoner_ids)
            .fetch_all(db)
            .await
            .unwrap();
        let mut nested_map= HashMap::new();

        for participant in query_results {
            // Insert a new HashMap for this summoner_id if it doesn't already exist
            nested_map
                .entry(participant.summoner_id)
                .or_insert_with(HashMap::new)
                // Insert the participant data into the inner HashMap by champion_id
                .insert(participant.champion_id, participant);
        }
        Ok(nested_map)
    }


    pub async fn get_champions_for_summoner(
        db: &PgPool,
        summoner_id: i32,
        filters: MatchFiltersSearch,
    ) -> AppResult<Vec<LolSummonerChampionPage>> {
        let start_date = parse_date(filters.start_date.clone());
        let end_date = parse_date(filters.end_date.clone());

        let mut query = QueryBuilder::new(r#"
            SELECT
                lmp.champion_id,
                count(lmp.lol_match_id) as total_matches,
                sum(CASE WHEN lmp.won THEN 1 ELSE 0 END) AS total_wins,
                avg(lmp.kda) as avg_kda,
                avg(lmp.kill_participation) as avg_kill_participation,
                avg(lmp.kills) as avg_kills,
                avg(lmp.deaths) as avg_deaths,
                avg(lmp.assists) as avg_assists,
                avg(lmp.gold_earned) as avg_gold_earned,
                avg(lmp.cs) as avg_cs,
                avg(lmp.damage_dealt_to_champions) as avg_damage_dealt_to_champions,
                avg(lmp.damage_taken) as avg_damage_taken,
                sum(lmp.double_kills) AS total_double_kills,
                sum(lmp.triple_kills) AS total_triple_kills,
                sum(lmp.quadra_kills) AS total_quadra_kills,
                sum(lmp.penta_kills) AS total_penta_kills
            FROM lol_match_participants as lmp
                     INNER JOIN  (SELECT id, queue_id, match_end FROM lol_matches) AS lm ON lm.id = lmp.lol_match_id
            WHERE lmp.summoner_id = "#);
        query.push_bind(summoner_id);
        if let Some(champion_id) = filters.champion_id {
            query.push(" AND lmp.champion_id = ");
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
        query.push(" GROUP BY lmp.champion_id ORDER BY total_matches DESC;");
        let results = query.build_query_as::<LolSummonerChampionResult>().fetch_all(db).await.unwrap();
       Ok(results.into_iter().map(|r|{
           let total_lose= r.total_matches - r.total_wins;
           let win_rate = round_to_2_decimal_places((r.total_wins as f64 / r.total_matches as f64) * 100.0);
           LolSummonerChampionPage{
                champion_id: r.champion_id,
                total_matches: r.total_matches,
                total_wins: r.total_wins,
                total_lose,
                win_rate,
                avg_kda: round_to_2_decimal_places(r.avg_kda.to_f64().unwrap_or_default()),
                avg_kill_participation: round_to_2_decimal_places(r.avg_kill_participation.to_f64().unwrap_or_default()),
                avg_kills: round_to_2_decimal_places(r.avg_kills.to_f64().unwrap_or_default()),
                avg_deaths: round_to_2_decimal_places(r.avg_deaths.to_f64().unwrap_or_default()),
                avg_assists: round_to_2_decimal_places(r.avg_assists.to_f64().unwrap_or_default()),
                avg_gold_earned: round_to_2_decimal_places(r.avg_gold_earned.to_f64().unwrap_or_default()),
                avg_cs: round_to_2_decimal_places(r.avg_cs.to_f64().unwrap_or_default()),
                avg_damage_dealt_to_champions: round_to_2_decimal_places(r.avg_damage_dealt_to_champions.to_f64().unwrap_or_default()),
                avg_damage_taken: round_to_2_decimal_places(r.avg_damage_taken.to_f64().unwrap_or_default()),
                total_double_kills: r.total_double_kills,
                total_triple_kills: r.total_triple_kills,
                total_quadra_kills: r.total_quadra_kills,
                total_penta_kills: r.total_penta_kills,
           }
       }).collect::<Vec<_>>())
    }


    pub async fn get_match_participant_for_matches_page(
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
        let ag_build = aggregate_query.build_query_as::<LolMatchParticipantMatchesQueryAggregateResult>();

        let aggregate_result = ag_build
            .fetch_one(db)
            .await
            .unwrap();
        let results = participant_query
            .build_query_as::<LolMatchParticipantMatchesQueryResult>()
            .fetch_all(db)
            .await
            .unwrap();


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
            LolMatchDefaultParticipantMatchesPage {
                summoner_id: row.summoner_id,
                match_id: row.lol_match_id,
                riot_match_id: row.riot_match_id,
                platform: row.platform.unwrap_or_default(),
                match_ended_since,
                match_duration: match_duration_str,
                queue: Queue::try_from(row.lol_match_queue_id.unwrap_or_default() as u16)
                    .unwrap()
                    .to_string(),
                champion_id: row.champion_id,
                champ_level: row.champ_level,
                won: row.won,
                kda,
                kills: row.kills,
                deaths: row.deaths,
                assists: row.assists,
                kill_participation,
                summoner_spell1_id: row.summoner_spell1_id.unwrap_or_default(),
                summoner_spell2_id: row.summoner_spell2_id.unwrap_or_default(),
                perk_primary_selection_id: row.perk_primary_selection_id.unwrap_or_default(),
                perk_sub_style_id: row.perk_sub_style_id.unwrap_or_default(),
                item0_id: row.item0_id.unwrap_or_default(),
                item1_id: row.item1_id.unwrap_or_default(),
                item2_id: row.item2_id.unwrap_or_default(),
                item3_id: row.item3_id.unwrap_or_default(),
                item4_id: row.item4_id.unwrap_or_default(),
                item5_id: row.item5_id.unwrap_or_default(),
                item6_id: row.item6_id.unwrap_or_default(),
                participants: vec![],
            }
        }).collect::<Vec<_>>();
        let total_pages = (matches_result_info.total_matches as f64 / per_page as f64).ceil() as i32;

        // Fetch participants for the collected match_ids

        let participants = if !matches_ids.is_empty() {
            let participant_rows = sqlx::query_as::<_, LolMatchParticipantMatchesChildQueryResult>(
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
                .map(|row| LolMatchParticipantMatchesPage {
                    team_id: row.team_id,
                    lol_match_id: row.lol_match_id,
                    summoner_id: row.summoner_id,
                    summoner_name: row.summoner_name,
                    champion_id: row.champion_id,
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

    pub async fn bulk_insert(db: &sqlx::PgPool, participants: &[TempParticipant]) -> AppResult<()> {
        // Collect all fields into vectors
        let (champion_ids, summoner_ids, match_ids, summoner_spell1_ids, summoner_spell2_ids, team_ids, won_flags, champ_levels, kill_participations, kdas, killss, deathss, assistss, damage_dealt_to_championss, damage_takens, gold_earneds, wards_placeds, css, css_per_minute, double_kills, triple_kills, quadra_kills, penta_kills): (
            Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>
        ) = participants.iter().map(|p| {
            (
                p.champion_id,
                p.summoner_id,
                p.lol_match_id,
                p.summoner_spell1_id,
                p.summoner_spell2_id,
                p.team_id,
                p.won,
                p.champ_level,
                p.kill_participation,
                p.kda,
                p.kills,
                p.deaths,
                p.assists,
                p.damage_dealt_to_champions,
                p.damage_taken,
                p.gold_earned,
                p.wards_placed,
                p.cs,
                p.cs_per_minute,
                p.double_kills,
                p.triple_kills,
                p.quadra_kills,
                p.penta_kills,
            )
        }).unzip_n();

        let perk_ids:
            (
                Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>
            ) = participants.iter().map(|p| {
            (
                p.perk_defense_id,
                p.perk_flex_id,
                p.perk_offense_id,
                p.perk_primary_style_id,
                p.perk_sub_style_id,
                p.perk_primary_selection_id,
                p.perk_primary_selection1_id,
                p.perk_primary_selection2_id,
                p.perk_primary_selection3_id,
                p.perk_sub_selection1_id,
                p.perk_sub_selection2_id,
            )
        }).unzip_n();

        let item_ids: (Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>) = participants.iter().map(|p| {
            (
                p.item0_id,
                p.item1_id,
                p.item2_id,
                p.item3_id,
                p.item4_id,
                p.item5_id,
                p.item6_id,
            )
        }).unzip_n();

        let sql = r#"
            INSERT INTO lol_match_participants (
                champion_id,
                summoner_id,
                lol_match_id,
                summoner_spell1_id,
                summoner_spell2_id,
                team_id,
                won,
                champ_level,
                kill_participation,
                kda,
                kills,
                deaths,
                assists,
                damage_dealt_to_champions,
                damage_taken,
                gold_earned,
                wards_placed,
                cs,
                cs_per_minute,
                double_kills,
                triple_kills,
                quadra_kills,
                penta_kills,
                perk_defense_id,
                perk_flex_id,
                perk_offense_id,
                perk_primary_style_id,
                perk_sub_style_id,
                perk_primary_selection_id,
                perk_primary_selection1_id,
                perk_primary_selection2_id,
                perk_primary_selection3_id,
                perk_sub_selection1_id,
                perk_sub_selection2_id,
                item0_id,
                item1_id,
                item2_id,
                item3_id,
                item4_id,
                item5_id,
                item6_id
            )
            SELECT * FROM UNNEST (
                $1::INT[],
                $2::INT[],
                $3::INT[],
                $4::INT[],
                $5::INT[],
                $6::INT[],
                $7::BOOL[],
                $8::INT[],
                $9::FLOAT8[],
                $10::FLOAT8[],
                $11::INT[],
                $12::INT[],
                $13::INT[],
                $14::INT[],
                $15::INT[],
                $16::INT[],
                $17::INT[],
                $18::INT[],
                $19::FLOAT8[],
                $20::INT[],
                $21::INT[],
                $22::INT[],
                $23::INT[],
                $24::INT[],
                $25::INT[],
                $26::INT[],
                $27::INT[],
                $28::INT[],
                $29::INT[],
                $30::INT[],
                $31::INT[],
                $32::INT[],
                $33::INT[],
                $34::INT[],
                $35::INT[],
                $36::INT[],
                $37::INT[],
                $38::INT[],
                $39::INT[],
                $40::INT[],
                $41::INT[]
            );
        "#;

        sqlx::query(sql)
            .bind(&champion_ids)
            .bind(&summoner_ids)
            .bind(&match_ids)
            .bind(&summoner_spell1_ids)
            .bind(&summoner_spell2_ids)
            .bind(&team_ids)
            .bind(&won_flags)
            .bind(&champ_levels)
            .bind(&kill_participations)
            .bind(&kdas)
            .bind(&killss)
            .bind(&deathss)
            .bind(&assistss)
            .bind(&damage_dealt_to_championss)
            .bind(&damage_takens)
            .bind(&gold_earneds)
            .bind(&wards_placeds)
            .bind(&css)
            .bind(&css_per_minute)
            .bind(&double_kills)
            .bind(&triple_kills)
            .bind(&quadra_kills)
            .bind(&penta_kills)
            .bind(&perk_ids.0)
            .bind(&perk_ids.1)
            .bind(&perk_ids.2)
            .bind(&perk_ids.3)
            .bind(&perk_ids.4)
            .bind(&perk_ids.5)
            .bind(&perk_ids.6)
            .bind(&perk_ids.7)
            .bind(&perk_ids.8)
            .bind(&perk_ids.9)
            .bind(&perk_ids.10)
            .bind(&item_ids.0)
            .bind(&item_ids.1)
            .bind(&item_ids.2)
            .bind(&item_ids.3)
            .bind(&item_ids.4)
            .bind(&item_ids.5)
            .bind(&item_ids.6)
            .execute(db)
            .await?;

        Ok(())
    }
}


fn format_duration_since(dt: NaiveDateTime) -> String {
    let match_end = dt.and_utc();
    let now = Utc::now();
    let duration = now.signed_duration_since(match_end);

    if duration.num_days() > 0 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{} minutes ago", duration.num_minutes())
    } else {
        format!("{} seconds ago", duration.num_seconds())
    }
}