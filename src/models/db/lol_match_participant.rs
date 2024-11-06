use crate::apis::MatchFiltersSearch;
use crate::consts::Queue;
use crate::error_template::{AppError, AppResult};
use crate::models::entities::lol_match_participant::{LolMatchDefaultParticipantMatchesPage, LolMatchParticipant, LolMatchParticipantMatchesDetailPage, LolMatchParticipantMatchesPage};
use crate::models::update::summoner_matches::TempParticipant;
use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::{Duration, NaiveDateTime, Utc};
use itertools::Itertools;
use leptos::prelude::BindAttribute;
use sqlx::{Execute, FromRow, PgPool, QueryBuilder, Row};
use unzip_n::unzip_n;
use crate::components::summoner_matches_page::{GetSummonerMatchesResult, MatchesResultInfo};

unzip_n!(19);
unzip_n!(11);
unzip_n!(7);


#[derive(FromRow)]
struct LolMatchParticipantDetailsQueryResult {
    id: i32,
    lol_match_id: i32,
    summoner_id: i32,
    summoner_name: String,
    summoner_tag_line: String,
    summoner_platform: String,
    summoner_icon_id: i32,
    summoner_level: i64,
    champion_id: i32,
    team_id: i32,
    won: bool,
    kills: i32,
    deaths: i32,
    assists: i32,
    champ_level: i32,
    kda: Option<BigDecimal>,
    kill_participation: Option<BigDecimal>,
    damage_dealt_to_champions: i32,
    damage_taken: i32,
    gold_earned: i32,
    wards_placed: i32,
    cs: i32,
    summoner_spell1_id: Option<i32>,
    summoner_spell2_id: Option<i32>,
    perk_defense_id: Option<i32>,
    perk_flex_id: Option<i32>,
    perk_offense_id: Option<i32>,
    perk_primary_style_id: Option<i32>,
    perk_sub_style_id: Option<i32>,
    perk_primary_selection_id: Option<i32>,
    perk_primary_selection1_id: Option<i32>,
    perk_primary_selection2_id: Option<i32>,
    perk_primary_selection3_id: Option<i32>,
    perk_sub_selection1_id: Option<i32>,
    perk_sub_selection2_id: Option<i32>,
    item0_id: Option<i32>,
    item1_id: Option<i32>,
    item2_id: Option<i32>,
    item3_id: Option<i32>,
    item4_id: Option<i32>,
    item5_id: Option<i32>,
    item6_id: Option<i32>,
}


#[derive(FromRow)]
struct LolMatchParticipantMatchesQueryAggregateResult{
    #[allow(dead_code)]
    total_count: Option<i64>,
    total_wins: Option<i64>,
    avg_kills: Option<BigDecimal>,
    avg_deaths: Option<BigDecimal>,
    avg_assists: Option<BigDecimal>,
    avg_kda: Option<BigDecimal>,
    avg_kill_participation: Option<BigDecimal>,
}


#[derive(FromRow)]
struct LolMatchParticipantMatchesQueryResult {
    #[allow(dead_code)]
    id: i32,
    lol_match_id: i32,
    riot_match_id: String,
    platform: Option<String>,
    champion_id: i32,
    summoner_id: i32,
    summoner_spell1_id: Option<i32>,
    summoner_spell2_id: Option<i32>,
    #[allow(dead_code)]
    team_id: i32,
    won: bool,
    champ_level: i32,
    kill_participation: Option<BigDecimal>,
    kda: Option<BigDecimal>,
    kills: i32,
    deaths: i32,
    assists: i32,
    perk_primary_selection_id: Option<i32>,
    perk_sub_style_id: Option<i32>,
    item0_id: Option<i32>,
    item1_id: Option<i32>,
    item2_id: Option<i32>,
    item3_id: Option<i32>,
    item4_id: Option<i32>,
    item5_id: Option<i32>,
    item6_id: Option<i32>,
    lol_match_queue_id: Option<i32>,
    lol_match_match_end: Option<NaiveDateTime>,
    lol_match_match_duration: Option<i32>,
}

#[derive(FromRow)]
pub struct LolMatchParticipantMatchesChildQueryResult {
    pub team_id: i32,
    pub lol_match_id: i32,
    pub summoner_id: i32,
    pub summoner_name: String,
    pub champion_id: i32,
    pub summoner_tag_line: String,
    pub summoner_platform: String,
}

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


    pub async fn get_match_participant_for_matches_page(
        db: &PgPool,
        summoner_id: i32,
        page: i32,
        filters: MatchFiltersSearch,
    ) -> AppResult<GetSummonerMatchesResult> {
        let start_date = filters.start_date.as_deref().and_then(|s| {
            if s.is_empty() {
                None
            } else {
                NaiveDateTime::parse_from_str(&format!("{} 00:00:00", s), "%Y-%m-%d %H:%M:%S").ok()
            }
        });
        let end_date = filters.end_date.as_deref().and_then(|s| {
            if s.is_empty() {
                None
            } else {
                NaiveDateTime::parse_from_str(&format!("{} 00:00:00", s), "%Y-%m-%d %H:%M:%S").ok()
            }
        });
        let  per_page = 20;
        let  offset = (page.max(1) - 1) * per_page;
        let mut aggregate_query = QueryBuilder::new(r#"
            SELECT
                count(lmp.lol_match_id) as total_count,
                count(case when     lmp.won then 1 end) as total_wins,
                avg(lmp.kills)  as avg_kills,
                avg(lmp.deaths)  as avg_deaths,
                avg(lmp.assists)  as avg_assists,
                avg(lmp.kda)  as avg_kda,
                avg(lmp.kill_participation) as avg_kill_participation
            FROM lol_match_participants as lmp
            INNER JOIN lol_matches as lm ON lm.id = lmp.lol_match_id
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
            INNER JOIN lol_matches as lm ON lm.id = lmp.lol_match_id
            WHERE
                lmp.summoner_id =
        "#);
        participant_query.push_bind(summoner_id);


        if let Some(champion_id) = filters.champion_id{
            let sql_filter = " AND lmp.champion_id = ";
            participant_query.push(&sql_filter);
            participant_query.push_bind(champion_id);
            aggregate_query.push(&sql_filter);
            aggregate_query.push_bind(champion_id);
        }
        if let Some(queue_id) = filters.queue_id{
            let sql_filter = " AND lm.queue_id = ";
            participant_query.push(&sql_filter);
            participant_query.push_bind(queue_id);
            aggregate_query.push(&sql_filter);
            aggregate_query.push_bind(queue_id);
        }

        if let Some(start_date) = start_date{
            let sql_filter = " AND lm.match_end >= ";
            participant_query.push(&sql_filter);
            participant_query.push_bind(start_date);
            aggregate_query.push(&sql_filter);
            aggregate_query.push_bind(start_date);
        }
        if let Some(end_date) = end_date{
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
        let results =  participant_query
            .build_query_as::<LolMatchParticipantMatchesQueryResult>()
            .fetch_all(db)
            .await
            .unwrap();


        let matches_result_info = {
            let total_matches = aggregate_result.total_count.unwrap_or_default() as i32;
            let total_wins = aggregate_result.total_wins.unwrap_or_default() as i32;
            let total_losses = total_matches - total_wins;
            let round_2 = |x: f64| (x * 100.0).round() / 100.0;
            MatchesResultInfo{
                total_matches,
                total_wins,
                total_losses,
                avg_kills:round_2(aggregate_result.avg_kills.clone().unwrap_or_default().to_f64().unwrap_or_default()),
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

        Ok(GetSummonerMatchesResult { matches, total_pages: total_pages as i64, matches_result_info})
    }

    pub async fn bulk_insert(db: &sqlx::PgPool, participants: &[TempParticipant]) -> AppResult<()> {
        // Collect all fields into vectors
        let (champion_ids, summoner_ids, match_ids, summoner_spell1_ids, summoner_spell2_ids, team_ids, won_flags, champ_levels, kill_participations, kdas, killss, deathss, assistss, damage_dealt_to_championss, damage_takens, gold_earneds, wards_placeds, css, stats_json): (
            Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>
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
                serde_json::to_value(&p.stats).unwrap(),
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
                stats,
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
                $19::JSONB[],
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
                $37::INT[]
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
            .bind(&stats_json)
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