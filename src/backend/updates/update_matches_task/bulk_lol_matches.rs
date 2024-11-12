use crate::backend::ssr::AppResult;
use crate::backend::updates::update_matches_task::LolMatchNotUpdated;
use crate::utils::version_to_major_minor;
use riven::models::match_v5::Match;
use sqlx::PgPool;

pub async fn bulk_trashed_matches(db: &PgPool, matches: Vec<(Match, LolMatchNotUpdated)>) -> AppResult<()> {
    let match_ids = matches.iter().map(|(match_, db_match)| db_match.id).collect::<Vec<i32>>();
    let sql = r"
        UPDATE lol_matches
        SET
            trashed = true,
            updated = true
        WHERE id = ANY($1)
        RETURNING id;
        ";
    sqlx::query(sql)
        .bind(match_ids)
        .fetch_all(db)
        .await?;
    Ok(())
}


pub async fn bulk_update_matches(db: &sqlx::PgPool, matches: Vec<(Match, LolMatchNotUpdated)>) -> AppResult<()> {
    let match_ids = matches.iter().map(|(x, _)| x.metadata.match_id.clone()).collect::<Vec<String>>();
    let match_creations = matches.iter().map(|(x, _)| chrono::DateTime::from_timestamp_millis(x.info.game_start_timestamp).unwrap_or_default()).collect::<Vec<_>>();
    let match_ends = matches.iter().map(|(x, _)| chrono::DateTime::from_timestamp_millis(x.info.game_end_timestamp.unwrap_or_default()).unwrap_or_default()).collect::<Vec<_>>();
    let match_durations = matches.iter().map(|(x, _)| x.info.game_duration as i32).collect::<Vec<i32>>();
    let queue_ids = matches.iter().map(|(x, _)| x.info.queue_id.0 as i32).collect::<Vec<i32>>();
    let map_ids = matches.iter().map(|(x, _)| x.info.map_id.0 as i32).collect::<Vec<i32>>();
    let versions = matches.iter().map(|(x, _)| version_to_major_minor(x.info.game_version.clone())).collect::<Vec<String>>();
    let modes = matches.iter().map(|(x, _)| x.info.game_mode.to_string()).collect::<Vec<String>>();
    let sql = r"
        UPDATE lol_matches
        SET
            match_creation = data.match_creation,
            match_end = data.match_end,
            match_duration = data.match_duration,
            queue_id = data.queue_id,
            map_id = data.map_id,
            game_mode = data.game_mode,
            version = data.version,
            updated = true
        FROM (
            SELECT
                UNNEST($1::VARCHAR(17)[]) AS match_id,
                UNNEST($2::TIMESTAMP[]) AS match_creation,
                UNNEST($3::TIMESTAMP[]) AS match_end,
                UNNEST($4::INT[]) AS match_duration,
                UNNEST($5::INT[]) AS queue_id,
                UNNEST($6::INT[]) AS map_id,
                UNNEST($7::VARCHAR(15)[]) AS game_mode,
                UNNEST($8::VARCHAR(5)[]) AS version
        ) AS data
        WHERE lol_matches.match_id = data.match_id;
        ";
    sqlx::query(sql)
        .bind(match_ids)
        .bind(match_creations)
        .bind(match_ends)
        .bind(match_durations)
        .bind(queue_ids)
        .bind(map_ids)
        .bind(modes)
        .bind(versions)
        .fetch_all(db)
        .await?;

    Ok(())
}