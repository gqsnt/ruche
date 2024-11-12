use crate::backend::ssr::{AppResult, Id};
use crate::backend::updates::update_matches_task::TempSummoner;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub async fn bulk_update_summoners(db: &sqlx::PgPool, summoners: &[TempSummoner]) -> AppResult<()> {
    let game_names = summoners.iter().map(|x| x.game_name.clone()).collect::<Vec<String>>();
    let tag_lines = summoners.iter().map(|x| x.tag_line.clone()).collect::<Vec<String>>();
    let puuids = summoners.iter().map(|x| x.puuid.clone()).collect::<Vec<String>>();
    let platforms = summoners.iter().map(|x| x.platform.to_string()).collect::<Vec<String>>();
    let summoner_levels = summoners.iter().map(|x| x.summoner_level).collect::<Vec<i64>>();
    let profile_icon_ids = summoners.iter().map(|x| x.profile_icon_id).collect::<Vec<i32>>();
    let updated_ats = summoners.iter().map(|x| x.updated_at).collect::<Vec<DateTime<Utc>>>();

    let sql = r"
        UPDATE summoners
        SET
            game_name = data.game_name,
            tag_line = data.tag_line,
            platform = data.platform,
            summoner_level = data.summoner_level,
            profile_icon_id = data.profile_icon_id,
            updated_at = data.updated_at
        FROM (
            SELECT
                unnest($1::VARCHAR(16)[]) AS game_name,
                unnest($2::VARCHAR(5)[]) AS tag_line,
                unnest($3::VARCHAR(78)[]) AS puuid,
                unnest($4::VARCHAR(4)[]) AS platform,
                unnest($5::INT[]) AS summoner_level,
                unnest($6::INT[]) AS profile_icon_id,
                unnest($7::TIMESTAMP[]) AS updated_at
        ) AS data
        WHERE summoners.puuid = data.puuid;
        ";

    sqlx::query(sql)
        .bind(game_names)
        .bind(tag_lines)
        .bind(puuids)
        .bind(platforms)
        .bind(summoner_levels)
        .bind(profile_icon_ids)
        .bind(updated_ats)
        .execute(db)
        .await?;

    Ok(())
}

pub async fn bulk_insert_summoners(db: &sqlx::PgPool, summoners: &[TempSummoner]) -> AppResult<HashMap<String, i32>> {
    let game_names = summoners.iter().map(|x| x.game_name.clone()).collect::<Vec<String>>();
    let tag_lines = summoners.iter().map(|x| x.tag_line.clone()).collect::<Vec<String>>();
    let puuids = summoners.iter().map(|x| x.puuid.clone()).collect::<Vec<String>>();
    let platforms = summoners.iter().map(|x| x.platform.to_string()).collect::<Vec<String>>();
    let summoner_levels = summoners.iter().map(|x| x.summoner_level).collect::<Vec<i64>>();
    let profile_icon_ids = summoners.iter().map(|x| x.profile_icon_id).collect::<Vec<i32>>();
    let updated_ats = summoners.iter().map(|x| x.updated_at).collect::<Vec<DateTime<Utc>>>();
    let sql = r"
        INSERT INTO
            summoners
            (
                game_name,
                tag_line,
                puuid,
                platform,
                summoner_level,
                profile_icon_id,
                updated_at
            ) SELECT * FROM UNNEST (
                $1::VARCHAR(16)[],
                $2::VARCHAR(5)[],
                $3::VARCHAR(78)[],
                $4::VARCHAR(4)[],
                $5::INT[],
                $6::INT[],
                $7::TIMESTAMP[]
            )
            ON CONFLICT (puuid)
            DO UPDATE SET
                game_name = EXCLUDED.game_name,
                tag_line = EXCLUDED.tag_line,
                platform = EXCLUDED.platform,
                summoner_level = EXCLUDED.summoner_level,
                profile_icon_id = EXCLUDED.profile_icon_id,
                updated_at = EXCLUDED.updated_at
            WHERE summoners.updated_at < EXCLUDED.updated_at
            returning id;
        ";
    let rows = sqlx::query_as::<_, Id>(sql)
        .bind(game_names)
        .bind(tag_lines)
        .bind(puuids)
        .bind(platforms)
        .bind(summoner_levels)
        .bind(profile_icon_ids)
        .bind(updated_ats)
        .fetch_all(db)
        .await?;
    Ok(rows.into_iter().enumerate().map(|(index, r)| (summoners.get(index).unwrap().puuid.clone(), r.id)).collect::<HashMap<String, i32>>())
}