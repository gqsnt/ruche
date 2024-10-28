use crate::error_template::AppResult;
use crate::models::entities::lol_match_participant::{LolMatchDefaultParticipantMatchesPage, LolMatchParticipant, LolMatchParticipantMatchesPage, LolMatchParticipantStats};
use crate::models::update::summoner_matches::TempParticipant;
use unzip_n::unzip_n;
use crate::lol_static::{get_champion_by_id, get_champion_by_name, get_item, get_perk, get_queue, get_summoner_spell, CHAMPIONS, QUEUES};
use itertools::Itertools;
use bigdecimal::ToPrimitive;
unzip_n!(14);
unzip_n!(11);
unzip_n!(7);

impl LolMatchParticipant {
    pub async fn get_match_participant_for_matches_page(db: &sqlx::PgPool, summoner_id: i32, page:i64) -> AppResult<(Vec<LolMatchDefaultParticipantMatchesPage>, i64)> {
        let per_page = 10;
        let total_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM lol_match_participants
            INNER JOIN lol_matches ON lol_matches.id = lol_match_participants.lol_match_id
            WHERE lol_match_participants.summoner_id = $1;",
            summoner_id
        )
            .fetch_one(db)
            .await?;
        let total_pages = (total_count.unwrap() + per_page - 1) / per_page;

        let mut matches= sqlx::query!(
            "SELECT
                lol_match_participants.id,
                lol_match_participants.lol_match_id,
                lol_match_participants.champion_id,
                lol_match_participants.summoner_id,
                lol_match_participants.summoner_spell1_id,
                lol_match_participants.summoner_spell2_id,
                lol_match_participants.team_id,
                lol_match_participants.won,
                lol_match_participants.champ_level,
                lol_match_participants.kill_participation,
                lol_match_participants.kda,
                lol_match_participants.kills,
                lol_match_participants.deaths,
                lol_match_participants.assists,
                lol_match_participants.perk_primary_selection_id,
                lol_match_participants.perk_sub_style_id,
                lol_match_participants.item0_id,
                lol_match_participants.item1_id,
                lol_match_participants.item2_id,
                lol_match_participants.item3_id,
                lol_match_participants.item4_id,
                lol_match_participants.item5_id,
                lol_match_participants.item6_id,
                lol_matches.queue_id as lol_match_queue_id,
                lol_matches.match_end as lol_match_match_end,
                lol_matches.match_duration as lol_match_match_duration
            FROM lol_match_participants
            INNER JOIN lol_matches ON lol_matches.id = lol_match_participants.lol_match_id
            WHERE lol_match_participants.summoner_id = $1
            ORDER BY lol_matches.match_end DESC LIMIT 10 OFFSET $2;",
            summoner_id,
            page * 10
        )
            .map(|row|{
                let match_duration =  chrono::Duration::seconds(row.lol_match_match_duration.unwrap() as i64);
                let match_duration_str =  format!("{:02}:{:02}:{:02}", match_duration.num_hours(), match_duration.num_minutes()%60, match_duration.num_seconds() %60);
                // convert match _end to thing like "2 hours ago"
                let match_ended_since: String = {
                    let match_end = row.lol_match_match_end.unwrap();
                    let now = chrono::Utc::now();
                    let duration = now.signed_duration_since(match_end.and_utc());
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
                let kda= row.kda.to_f64().unwrap();
                let  kda = if kda.is_nan() {
                    0.0
                } else {
                    (kda *100.0 ).round() / 100.0
                };
                let  kill_participation = row.kill_participation.to_f64().unwrap();
                let  kill_participation = if kill_participation.is_nan() {
                    0.0
                } else {
                    (kill_participation *100.0 ).round() / 100.0
                };

                LolMatchDefaultParticipantMatchesPage{
                    summoner_id:row.summoner_id,
                    match_id: row.lol_match_id,
                    match_ended_since,
                    match_duration: match_duration_str,
                    queue: get_queue(row.lol_match_queue_id.unwrap()).description.unwrap_or_default(),
                    champion_id: row.champion_id,
                    champion_img_url: get_champion_by_id(row.champion_id).img_url,
                    champ_level:row.champ_level,
                    won: row.won,
                    kda,
                    kills: row.kills,
                    deaths: row.deaths,
                    assists: row.assists,
                    kill_participation,
                    summoner_spell1_id: row.summoner_spell1_id.unwrap_or_default(),
                    summoner_spell_img_url1: get_summoner_spell(row.summoner_spell1_id.unwrap_or_default()).unwrap_or_default().img_url,
                    summoner_spell2_id: row.summoner_spell2_id.unwrap_or_default(),
                    summoner_spell_img_url2: get_summoner_spell(row.summoner_spell2_id.unwrap_or_default()).unwrap_or_default().img_url,
                    perk_primary_selection_id: row.perk_primary_selection_id.unwrap_or_default(),
                    perk_primary_selection_url: get_perk(row.perk_primary_selection_id.unwrap_or_default()).unwrap_or_default().img_url,
                    perk_sub_style_id: row.perk_sub_style_id.unwrap_or_default(),
                    perk_sub_style_img_url: get_perk(row.perk_sub_style_id.unwrap_or_default()).unwrap_or_default().img_url,
                    item0_id: row.item0_id.unwrap_or_default(),
                    item0_img_url: get_item(row.item0_id.unwrap_or_default()).unwrap_or_default().img_url,
                    item1_id:  row.item1_id.unwrap_or_default(),
                    item1_img_url: get_item(row.item1_id.unwrap_or_default()).unwrap_or_default().img_url,
                    item2_id: row.item2_id.unwrap_or_default(),
                    item2_img_url: get_item(row.item2_id.unwrap_or_default()).unwrap_or_default().img_url,
                    item3_id: row.item3_id.unwrap_or_default(),
                    item3_img_url: get_item(row.item3_id.unwrap_or_default()).unwrap_or_default().img_url,
                    item4_id: row.item4_id.unwrap_or_default(),
                    item4_img_url: get_item(row.item4_id.unwrap_or_default()).unwrap_or_default().img_url,
                    item5_id: row.item5_id.unwrap_or_default(),
                    item5_img_url: get_item(row.item5_id.unwrap_or_default()).unwrap_or_default().img_url,
                    item6_id: row.item6_id.unwrap_or_default(),
                    item6_img_url: get_item(row.item6_id.unwrap_or_default()).unwrap_or_default().img_url,
                    participants: vec![],
                }
            })
            .fetch_all(db)
            .await?;

        let match_ids = matches.iter().map(|m| m.match_id).collect::<Vec<_>>();
        let participants = sqlx::query!("
            SELECT
                lol_match_participants.lol_match_id,
                lol_match_participants.summoner_id,
                lol_match_participants.champion_id,
                lol_match_participants.team_id,
                summoners.game_name as summoner_name,
                summoners.tag_line as summoner_tag_line,
                summoners.platform as summoner_platform
                FROM  lol_match_participants
                INNER JOIN summoners ON summoners.id = lol_match_participants.summoner_id
                WHERE lol_match_participants.lol_match_id = ANY($1);",
            &match_ids
        ).map(|row|{
            LolMatchParticipantMatchesPage{
                team_id: row.team_id,
                lol_match_id: row.lol_match_id,
                summoner_id: row.summoner_id,
                summoner_name: row.summoner_name,
                champion_id: row.champion_id,
                summoner_tag_line: row.summoner_tag_line,
                summoner_platform: row.summoner_platform,
                champion_img_url: get_champion_by_id(row.champion_id).img_url,
            }
        })
            .fetch_all(db)
            .await?;
        let participants = participants.into_iter().into_group_map_by(|p| p.lol_match_id).into_iter().collect::<Vec<_>>();
        for match_ in matches.iter_mut(){
            if let Some((match_id, participants)) = participants.iter().find(|p| p.0 == match_.match_id){
                match_.participants = participants.clone();
            }
        }

        Ok((matches, total_pages))
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