use std::str::FromStr;
use crate::error_template::{AppError, AppResult};
use crate::models::entities::lol_match_participant::{LolMatchDefaultParticipantMatchesPage, LolMatchParticipant, LolMatchParticipantMatchesPage, LolMatchParticipantStats};
use crate::models::update::summoner_matches::TempParticipant;
use unzip_n::unzip_n;
use itertools::Itertools;
use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use sqlx::{Execute, FromRow, PgPool, QueryBuilder, Row};
use crate::apis::MatchFiltersSearch;
use crate::consts::Queue;
use crate::models::db::Id;
use crate::models::entities::summoner::Summoner;

unzip_n!(14);
unzip_n!(11);
unzip_n!(7);


#[derive(FromRow)]
struct LolMatchParticipantMatchesQueryResult{
    total_count: Option<i64>,
    id: i32,
    lol_match_id: i32,
    champion_id: i32,
    summoner_id: i32,
    summoner_spell1_id: Option<i32>,
    summoner_spell2_id: Option<i32>,
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


impl LolMatchParticipant {
    pub async fn get_match_participant_for_matches_page(
        db: &PgPool,
        summoner_id: i32,
        page: i32,
        filters: MatchFiltersSearch,
    ) -> AppResult<(Vec<LolMatchDefaultParticipantMatchesPage>, i64)> {
        let start_date=  if let Some(start_date) = filters.start_date {
            if !start_date.is_empty() {
                Some(NaiveDateTime::parse_from_str(format!("{} 00:00:00", start_date).as_str(), "%Y-%m-%d %H:%M:%S").unwrap())
            } else {
                None
            }
        }else{
            None
        };
        let end_date = if let Some(end_date) = filters.end_date {
            if !end_date.is_empty() {
                Some(NaiveDateTime::parse_from_str(format!("{} 00:00:00", end_date).as_str(), "%Y-%m-%d %H:%M:%S").unwrap())
            } else {
                None
            }
        }else{
            None
        };
        let per_page = 20;
        let offset = (page.max(1) - 1) * per_page;


        let query = sqlx::query_as!(
            LolMatchParticipantMatchesQueryResult,
            "SELECT
                count(lol_match_participants.lol_match_id) over() as total_count,
                lol_match_participants.id,
                lol_match_participants.lol_match_id,
                lol_match_participants.champion_id,
                lol_match_participants.summoner_id,
                lol_match_participants.team_id,
                lol_match_participants.won,
                lol_match_participants.champ_level,
                lol_match_participants.kill_participation,
                lol_match_participants.kda,
                lol_match_participants.kills,
                lol_match_participants.deaths,
                lol_match_participants.assists,
                lol_match_participants.summoner_spell1_id,
                lol_match_participants.summoner_spell2_id,
                lol_match_participants.perk_primary_selection_id,
                lol_match_participants.perk_sub_style_id,
                lol_match_participants.item0_id,
                lol_match_participants.item1_id,
                lol_match_participants.item2_id,
                lol_match_participants.item3_id,
                lol_match_participants.item4_id,
                lol_match_participants.item5_id,
                lol_match_participants.item6_id,
                lol_matches.queue_id AS lol_match_queue_id,
                lol_matches.match_end AS lol_match_match_end,
                lol_matches.match_duration AS lol_match_match_duration
            FROM lol_match_participants
            INNER JOIN lol_matches ON lol_matches.id = lol_match_participants.lol_match_id
            AND lol_match_participants.summoner_id = $1
            AND ($2::INTEGER IS NULL OR lol_matches.queue_id = $2)
            AND ($3::INTEGER IS NULL OR lol_match_participants.champion_id = $3)
            AND ($4::TIMESTAMP IS NULL OR lol_matches.match_end >= $4)
            AND ($5::TIMESTAMP IS NULL OR lol_matches.match_end <= $5)
            ORDER BY lol_matches.match_end DESC
            LIMIT $6 OFFSET $7;",
            summoner_id,
            filters.queue_id,
            filters.champion_id,
            start_date,
            end_date,
            per_page as i64,
            offset as i64
        );
        //println!("{}\n$1:{:?}, $2:{}, $3:{}, $4:{}", query.sql(), matches_ids, summoner_id, per_page, offset);
        let mut total_matches = 0;
        let mut matches_ids = vec![];
        let mut matches =query.fetch_all(db).await?.into_iter().map(|row|{
            if total_matches == 0 {
                total_matches = row.total_count.unwrap_or_default();
            }
            matches_ids.push(row.lol_match_id);
            let match_duration = Duration::seconds(row.lol_match_match_duration.unwrap_or_default() as i64);
            let match_duration_str = format!(
                "{:02}:{:02}:{:02}",
                match_duration.num_hours(),
                match_duration.num_minutes() % 60,
                match_duration.num_seconds() % 60
            );

            // Calculate time since match ended
            let match_ended_since = {
                let match_end = row.lol_match_match_end.unwrap_or_default().and_utc();
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
            };

            // Safely handle floating point operations
            let kda = (row.kda.unwrap_or_default().to_f64().unwrap_or(0.0).max(0.0) * 100.0).round() / 100.0;
            let kill_participation = (row.kill_participation.unwrap_or_default().to_f64().unwrap_or(0.0).max(0.0) * 100.0).round();
            LolMatchDefaultParticipantMatchesPage {
                summoner_id: row.summoner_id,
                match_id: row.lol_match_id,
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
        let total_pages = (total_matches as f64 / per_page as f64).ceil() as i32;

        // Fetch participants for the collected match_ids
        let participants = if !matches_ids.is_empty() {
            sqlx::query!(
            "
            SELECT
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
            ORDER BY lol_match_participants.team_id ASC;
            ",
            &matches_ids
        )
                .fetch_all(db)
                .await
                .unwrap()
                .into_iter()
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

        Ok((matches, total_pages as i64))
    }

    pub async fn bulk_insert(db: &sqlx::PgPool, participants: &[TempParticipant]) -> AppResult<()> {
        // Collect all fields into vectors
        let (champion_ids, summoner_ids, match_ids, summoner_spell1_ids, summoner_spell2_ids, team_ids, won_flags,champ_levels,kill_participations,kdas,killss, deathss, assistss , stats_json): (
            Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>,Vec<_>, Vec<_>, Vec<_>,  Vec<_>, Vec<_>, Vec<_>
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
                $14::JSONB[],
                $15::INT[],
                $16::INT[],
                $17::INT[],
                $18::INT[],
                $19::INT[],
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
                $32::INT[]
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